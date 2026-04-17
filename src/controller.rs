use std::{sync::Arc, time::Duration};

use kube::{
    Api, Client,
    api::{Patch, PatchParams},
    runtime::{controller::Action, watcher},
};

use crate::{
    crd::{WatchRecord, WatchState},
    util,
};

pub struct Context {
    pub client: Client,
}

#[derive(Debug)]
pub struct ControllerError(pub String);

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// This is what kube-rs is desperately looking for!
impl std::error::Error for ControllerError {}

// Replace your original type alias with this:
pub type Error = ControllerError;

use futures::StreamExt;

pub fn create_controller(client: Client) {
    let controller = kube::runtime::Controller::new(
        Api::<WatchRecord>::default_namespaced(client.clone()),
        watcher::Config::default(),
    );
    let context = Arc::new(Context { client });

    tokio::spawn(async move {
        controller
            .run(reconciler, error_policy, context)
            .for_each(|_| async {})
            .await;
    });
}

pub async fn reconciler(obj: Arc<WatchRecord>, ctx: Arc<Context>) -> Result<Action, Error> {
    let wr_namespace = &obj
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());
    let anime = util::get_anime(&ctx.client, &obj).await.unwrap();
    let watched_eps = obj.spec.episodes_watched;
    let total_eps = anime.spec.total_episodes.expect("shouldn't happen");

    let status = if watched_eps == total_eps {
        Some(WatchState::Completed)
    } else if watched_eps < total_eps
        && !obj.status.as_ref().is_some_and(|x| {
            matches!(x.watch_state, WatchState::Dropped)
                || matches!(x.watch_state, WatchState::OnHold)
                || matches!(x.watch_state, WatchState::PlanToWatch)
        })
    {
        Some(WatchState::Watching)
    } else {
        obj.spec.status.clone().map(|x| x.into())
    };

    if let Some(status) = status {
        let patch = serde_json::json!({ "status": { "watchState": status } });

        Api::<WatchRecord>::namespaced(ctx.client.clone(), wr_namespace)
            .patch_status(
                obj.metadata.name.as_ref().unwrap(),
                &PatchParams::default(),
                &Patch::Merge(patch),
            )
            .await
            .unwrap();
    }

    Ok(Action::requeue(Duration::from_secs(1800)))
}

fn error_policy(_obj: Arc<WatchRecord>, _err: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5)) // Re-queue quickly on error
}
