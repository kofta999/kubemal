# KubeMAL

KubeMAL is a custom Kubernetes operator and admission webhook server written in Rust. It extends the Kubernetes API to function as a tracker for anime series, automatically fetching metadata from AniList and managing watch progress states through custom resources.

## Architecture

KubeMAL consists of three main components: a Mutating Webhook, a Validating Webhook, and an asynchronous Controller.

```mermaid
graph TD
    User[User / kubectl] -->|Applies CR| K8s[Kubernetes API Server]

    subgraph KubeMAL Application
        MutatingWebhook[Mutating Webhook]
        ValidatingWebhook[Validating Webhook]
        Controller[WatchRecord Controller]
    end

    K8s -->|CREATE Anime| MutatingWebhook
    MutatingWebhook -->|Query via GraphQL| AniList[AniList API]
    MutatingWebhook -->|Inject Metadata| K8s

    K8s -->|CREATE/UPDATE WatchRecord| ValidatingWebhook
    ValidatingWebhook -->|Validate Episodes| K8s

    K8s -->|Event Stream| Controller
    Controller -->|Update Status| K8s
```

## Features

- **Mutating Webhook (`/mutate`)**: Intercepts the creation of `Anime` custom resources. It takes the provided name, queries the AniList GraphQL API, and automatically populates the resource's spec with the English title, Japanese title, total episode count, and current airing status.
- **Validating Webhook (`/validate`)**: Intercepts the creation and updates of `WatchRecord` custom resources. It validates that the watched episode count is greater than zero and does not exceed the total episode count of the referenced `Anime`.
- **Reconciliation Controller**: Continuously watches `WatchRecord` resources. It calculates the correct `watchState` (Watching, Completed, Dropped, PlanToWatch, OnHold) based on the user's progress and updates the resource's status subresource.

## Custom Resources

### Anime

The `Anime` resource represents a specific show. Due to the mutating webhook, you only need to provide a basic name, and KubeMAL will populate the rest.

**Example Input:**

```yaml
apiVersion: kubemal.kofta.app/v1
kind: Anime
metadata:
  name: tonari-no-tenshi
spec: {}
```

### WatchRecord

The `WatchRecord` resource tracks a user's progress for a specific `Anime`.

**Example Input:**

```yaml
apiVersion: kubemal.kofta.app/v1
kind: WatchRecord
metadata:
  name: tonari-no-tenshi-watch-record
spec:
  username: kofta
  episodesWatched: 11
  score: 9
  animeRef:
    name: tonari-no-tenshi
```

## State Calculation Logic

The controller updates the `WatchRecord` status based on the following logic flow:

```mermaid
stateDiagram-v2
    direction TB
    [*] --> EvaluateEpisodes

    EvaluateEpisodes --> Completed: watched == total_episodes
    EvaluateEpisodes --> EvaluateStatusField: watched != total_episodes

    EvaluateStatusField --> Dropped: spec.status == Dropped
    EvaluateStatusField --> PlanToWatch: spec.status == PlanToWatch
    EvaluateStatusField --> OnHold: spec.status == OnHold
    EvaluateStatusField --> Watching: spec.status is None
```

## Prerequisites

To deploy KubeMAL to your cluster, you will need:

1.  A running Kubernetes cluster.
2.  `cert-manager` installed on your cluster (required for generating the self-signed TLS certificates used by the admission webhooks).

## Deployment

1.  **Apply the Custom Resource Definitions (CRDs):**

    ```bash
    kubectl apply -f kubernetes/crd/
    ```

2.  **Apply the Namespace, RBAC, and TLS configurations:**

    ```bash
    kubectl apply -f kubernetes/deploy/rbac.yaml
    kubectl apply -f kubernetes/deploy/tls.yaml
    ```

3.  **Deploy the KubeMAL application:**

    ```bash
    kubectl apply -f kubernetes/deploy/deployment.yaml
    kubectl apply -f kubernetes/deploy/service.yaml
    ```

4.  **Register the Webhooks:**
    _Ensure the KubeMAL pods are running before applying the webhook configuration._
    ```bash
    kubectl apply -f kubernetes/deploy/webhook.yaml
    ```

## Testing

You can test the functionality using the provided manifests in the `kubernetes/test/` directory.

1.  Create an Anime resource. You can describe it afterward to see the AniList data injected by the webhook.

    ```bash
    kubectl apply -f kubernetes/test/test-anime.yaml
    kubectl describe anime tonari-no-tenshi
    ```

2.  Create a WatchRecord.
    ```bash
    kubectl apply -f kubernetes/test/test-watchrecord.yaml
    kubectl describe watchrecord tonari-no-tenshi-watch-record
    ```
    Check the `Status` section of the WatchRecord to see the controller's computed watch state.
