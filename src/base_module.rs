use std::{collections::HashSet, rc::Rc, sync::Arc, thread};

use abi::{abi_stable, glib, gtk, log};
use abi_stable::external_types::crossbeam_channel::RSender;
use anyhow::{anyhow, Context, Result};
use dynisland_abi::module::UIServerCommand;
use glib::object::Cast;
use tokio::{
    runtime::Handle,
    sync::{
        broadcast::{Receiver, Sender},
        mpsc::UnboundedSender,
        Mutex,
    },
};

use crate::{
    activity_map::ActivityMap, dynamic_activity::DynamicActivity, dynamic_property::PropertyUpdate,
};

pub type Producer<T> = fn(module: &T);

/// A tokio runtime that performs a cleanup and stops when shutdown is called.
pub struct ProducerRuntime {
    handle: Mutex<Handle>,
    shutdown: Arc<Mutex<tokio::sync::mpsc::Sender<()>>>,
    cleanup_notifier: Sender<UnboundedSender<()>>,
}
impl Clone for ProducerRuntime {
    fn clone(&self) -> Self {
        Self {
            handle: Mutex::new(self.handle.blocking_lock().clone()),
            shutdown: self.shutdown.clone(),
            cleanup_notifier: self.cleanup_notifier.clone(),
        }
    }
}

impl Default for ProducerRuntime {
    fn default() -> Self {
        let (handle, shutdown) = Self::get_new_tokio_rt();
        let (cl_tx, _) = tokio::sync::broadcast::channel(32);
        Self {
            handle: Mutex::new(handle),
            shutdown: Arc::new(Mutex::new(shutdown)),
            cleanup_notifier: cl_tx,
        }
    }
}

impl ProducerRuntime {
    pub fn new() -> Self {
        Self::default()
    }
    /// Get an handle to the tokio runtime
    pub fn handle(&self) -> Handle {
        self.handle.blocking_lock().clone()
    }
    /// Starts a new runtime, if the runtime is still running, it will stop without calling the cleanup_notifier
    pub async fn reset(&self) {
        let (handle, shutdown) = Self::get_new_tokio_rt();
        *self.handle.lock().await = handle;
        *self.shutdown.lock().await = shutdown;
    }
    /// Starts a new runtime, if the runtime is still running, it will stop without calling the cleanup_notifier
    ///
    /// blocking
    pub fn reset_blocking(&self) {
        let (handle, shutdown) = Self::get_new_tokio_rt();
        *self.handle.blocking_lock() = handle;
        *self.shutdown.blocking_lock() = shutdown;
    }
    /// Get the cleanup notifier to receive a cleanup notification
    pub fn get_cleanup_notifier(&self) -> Receiver<UnboundedSender<()>> {
        self.cleanup_notifier.subscribe()
    }
    /// Shuts down the runtime after sending the cleanup notification and waiting for a confirmation.
    pub async fn shutdown(&self) {
        let num = self.cleanup_notifier.receiver_count();
        log::trace!("stopping producer runtime: {} cleanup receivers", num);
        let (res_tx, mut res_rx) = tokio::sync::mpsc::unbounded_channel();
        match self.cleanup_notifier.send(res_tx) {
            Ok(_) => {
                for i in 0..num {
                    log::trace!("waiting on cleanup {}", i + 1);
                    if res_rx.recv().await.is_none() {
                        //all of the remaining receivers already quit/crashed
                        break;
                    }
                }
            }
            Err(_) => {
                log::trace!("no cleanup needed");
            }
        }
        if self.shutdown.lock().await.send(()).await.is_err() {
            log::debug!("producer runtime has already quit")
        }
    }
    /// Shuts down the runtime after sending the cleanup notification and waiting for a confirmation.
    ///
    /// blocking
    pub fn shutdown_blocking(&self) {
        let num = self.cleanup_notifier.receiver_count();
        log::trace!("stopping producer runtime: {} cleanup receivers", num);
        let (res_tx, mut res_rx) = tokio::sync::mpsc::unbounded_channel();
        match self.cleanup_notifier.send(res_tx) {
            Ok(_) => {
                for i in 0..num {
                    log::trace!("waiting on cleanup {}", i + 1);
                    if res_rx.blocking_recv().is_none() {
                        //all of the remaining receivers already quit/crashed
                        break;
                    }
                }
            }
            Err(_) => {
                log::trace!("no cleanup needed");
            }
        }
        if self.shutdown.blocking_lock().blocking_send(()).is_err() {
            log::debug!("producer runtime has already quit")
        }
    }
    fn get_new_tokio_rt() -> (Handle, tokio::sync::mpsc::Sender<()>) {
        let (rt_send, rt_recv) =
            tokio::sync::oneshot::channel::<(Handle, tokio::sync::mpsc::Sender<()>)>();
        let (shutdown_send, mut shutdown_recv) = tokio::sync::mpsc::channel::<()>(1);
        std::thread::Builder::new()
            .name("dyn-producers".to_string() + &(rand::random::<u16>()).to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("idk tokio rt failed");
                let handle = rt.handle();
                rt_send
                    .send((handle.clone(), shutdown_send))
                    .expect("failed to send rt");
                rt.block_on(async {
                    shutdown_recv.recv().await;
                    log::info!(
                        "thread {} received shutdown",
                        thread::current().name().unwrap()
                    );
                }); //keep thread alive
                    // log::info!("shutting down runtime");
            })
            .expect("failed to spawn dyn-producers trhread");

        rt_recv.blocking_recv().expect("failed to receive rt")
    }
}

/// Base module logic
///
/// Handles the dynamic property update, keeps track and helps to register `DynamicActivity`s with the app.
///
/// Also keeps track of the producers to start when the config changes
pub struct BaseModule<T> {
    name: &'static str,
    app_send: RSender<UIServerCommand>,
    prop_send: UnboundedSender<PropertyUpdate>,
    registered_activities: Rc<Mutex<ActivityMap>>,
    registered_producers: Arc<Mutex<HashSet<Producer<T>>>>,
}

impl<T> Clone for BaseModule<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            app_send: self.app_send.clone(),
            prop_send: self.prop_send.clone(),
            registered_activities: self.registered_activities.clone(),
            registered_producers: self.registered_producers.clone(),
        }
    }
}

impl<T> BaseModule<T> {
    pub fn new(name: &'static str, app_send: RSender<UIServerCommand>) -> Self {
        let registered_activities = Rc::new(Mutex::new(ActivityMap::default()));
        let registered_producers = Arc::new(Mutex::new(HashSet::new()));
        let prop_send = Self::spawn_property_update_loop(&registered_activities);
        Self {
            name,
            app_send,
            prop_send,
            registered_activities,
            registered_producers,
        }
    }
    pub fn register_producer(&self, producer: Producer<T>) {
        self.registered_producers.blocking_lock().insert(producer);
    }

    pub fn registered_producers(&self) -> Arc<Mutex<HashSet<Producer<T>>>> {
        self.registered_producers.clone()
    }

    /// Register an activity with the app
    ///
    /// returns `Err` if the activity was already registered
    pub fn register_activity(&self, activity: DynamicActivity) -> Result<()> {
        let widget = activity.get_activity_widget();
        let id = activity.get_identifier();
        let activity = Rc::new(Mutex::new(activity));

        self.app_send
            .send(UIServerCommand::AddActivity {
                activity_id: id,
                widget: widget.upcast::<gtk::Widget>().into(),
            })
            .map_err(|err| anyhow!(err.to_string()))?;
        let mut reg = self.registered_activities.blocking_lock();
        reg.insert_activity(activity)
            .with_context(|| "failed to register activity")
    }

    /// Register an activity from an Rc<Mutex> with the app
    ///
    /// This should be used when the activity is registered/unregistered multiple times during the module lifetime.
    /// To avoid creating multiple instances of the same dynamic activity
    ///
    /// returns `Err` if the activity was already registered
    pub fn register_activity_rc(&self, activity: Rc<Mutex<DynamicActivity>>) -> Result<()> {
        let activity_lock = activity.blocking_lock();
        let widget = activity_lock.get_activity_widget();
        let id = activity_lock.get_identifier();
        drop(activity_lock);

        self.app_send
            .send(UIServerCommand::AddActivity {
                activity_id: id,
                widget: widget.upcast::<gtk::Widget>().into(),
            })
            .map_err(|err| anyhow!(err.to_string()))?;
        let mut reg = self.registered_activities.blocking_lock();
        reg.insert_activity(activity)
            .with_context(|| "failed to register activity")
    }

    /// Get a `Mutex` to the activity map
    pub fn registered_activities(&self) -> Rc<Mutex<ActivityMap>> {
        self.registered_activities.clone()
    }

    /// Unregister the activity with that name in the identifier
    ///
    /// # Arguments
    /// * `activity_name` - The name of the activity (activity_identifier.activity())
    ///
    /// does nothing if the activity wasn't registered
    pub fn unregister_activity(&self, activity_name: &str) {
        let activity_list = self
            .registered_activities()
            .blocking_lock()
            .list_activities();
        let identifier = activity_list.iter().find(|x| x.activity() == activity_name);
        if identifier.is_none() {
            log::debug!("activity {activity_name} isn't registered");
            return;
        }
        let identifier = identifier.unwrap().clone();
        self.app_send
            .send(UIServerCommand::RemoveActivity {
                activity_id: identifier,
            })
            .unwrap_or_else(|err| log::debug!("err: {err}"));

        match self
            .registered_activities
            .blocking_lock()
            .map
            .remove(activity_name)
        {
            Some(_) => {
                log::trace!("activity {activity_name} unregistered from base module");
            }
            None => {
                log::trace!("activity {activity_name} wasn't registered in base module");
            }
        }
    }

    fn spawn_property_update_loop(
        registered_activities: &Rc<Mutex<ActivityMap>>,
    ) -> UnboundedSender<PropertyUpdate> {
        //create ui property update channel
        let (prop_send, mut prop_recv) = tokio::sync::mpsc::unbounded_channel::<PropertyUpdate>();
        let activities = registered_activities.clone();
        glib::MainContext::default().spawn_local(async move {
            //start data consumer
            while let Some(res) = prop_recv.recv().await {
                if res.activity_id.activity() == "*" {
                    for activity in activities.lock().await.map.values() {
                        match activity.lock().await.get_subscribers(&res.property_name) {
                            core::result::Result::Ok(subs) => {
                                for sub in subs {
                                    sub(&*res.value);
                                }
                            }
                            Err(err) => {
                                log::error!("{}", err)
                            }
                        }
                    }
                } else {
                    match activities.lock().await.map.get(res.activity_id.activity()) {
                        Some(activity) => {
                            match activity.lock().await.get_subscribers(&res.property_name) {
                                core::result::Result::Ok(subs) => {
                                    for sub in subs {
                                        sub(&*res.value);
                                    }
                                }
                                Err(err) => {
                                    log::error!("{}", err)
                                }
                            }
                        }
                        None => {
                            // log::trace!("activity {} not found", res.activity_id);
                        }
                    }
                }
            }
        });
        prop_send
    }

    /// Get the channel to manually send property updates
    pub fn prop_send(&self) -> UnboundedSender<PropertyUpdate> {
        self.prop_send.clone()
    }
    /// Get the channel to communicate with the app
    pub fn app_send(&self) -> RSender<UIServerCommand> {
        self.app_send.clone()
    }
    /// Get the name of the module, should be the same as activity_identifier.activity()
    pub fn name(&self) -> &'static str {
        self.name
    }
}
