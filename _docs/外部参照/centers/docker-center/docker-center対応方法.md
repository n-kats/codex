# docker-center対応方法

docker-centerは、Dockerコンテナの登録、状態確認、操作を行うアプリケーションです。

この資料では、リポジトリ内のDocker関連ファイルをdocker-centerで利用できるようにするための実装方法を説明します。

## 起動スクリプト方式

リポジトリ内の起動スクリプトでコンテナを起動する方式です。Dockerfileからイメージをビルドする構成を主な例とし、compose定義をスクリプトから実行する構成も含みます。単一コンテナは`scripts/launch.sh`、個別に扱う複数コンテナは`scripts/launch_<サービス名>.sh`のようにスクリプトを分けます。

```text
<リポジトリ>/
├── docker/
│   └── service/
│       └── Dockerfile
└── scripts/
    └── launch.sh
```

複数のコンテナを個別に扱う構成例です。

```text
<リポジトリ>/
├── docker/
│   ├── api/
│   │   └── Dockerfile
│   └── worker/
│       └── Dockerfile
└── scripts/
    ├── launch_api.sh
    └── launch_worker.sh
```

ビルドコンテキストには、Dockerfileとビルドに必要なファイルだけを含む最小のディレクトリを指定します。例では`docker/service/`を使います。実行時のプロジェクトルートは、ビルドコンテキストと分けて`/workspace`へマウントします。

起動スクリプトには、docker-centerから`CONTAINER_NAME_PREFIX`を環境変数として渡します。各コンテナ名は`<CONTAINER_NAME_PREFIX><役割名>`とし、`docker run --name`へ指定します。役割名は固定し、同じ接頭辞の中で一意にします。単一コンテナには`service`、複数コンテナには`api`や`worker`などの役割名を使います。同じ起動スクリプトから複数を起動する場合も、この命名規則を使います。個別に扱う場合は起動スクリプトを分け、それぞれ対応する役割名を使います。

以下は単一コンテナ用の`scripts/launch.sh`の例です。

```bash
#!/usr/bin/env bash
set -e
: "${CONTAINER_NAME_PREFIX:?CONTAINER_NAME_PREFIXを設定してください}"
script_dir="$(realpath "$(dirname "$0")")" || exit 1
cd "$(dirname "$script_dir")" || exit 1
image_name="${IMAGE_NAME:-example-project-service}"
container_name="${CONTAINER_NAME_PREFIX}service"

if container_state="$(docker container inspect --format '{{.State.Status}}' "$container_name" 2>/dev/null)"; then
  if [ "$container_state" = "running" ]; then
    exit 0
  fi
  docker start "$container_name" >/dev/null
  exit 0
fi

docker build -t "$image_name" -f docker/service/Dockerfile docker/service
docker run -d --name "$container_name" \
  --user "$(id -u):$(id -g)" -v "$PWD:/workspace" -w /workspace \
  "$image_name" >/dev/null
```

同名のコンテナが稼働中なら起動済みとして終了し、停止中なら`docker start`で再開します。コンテナが存在しない場合にイメージをビルドして新規作成します。停止コマンドも同じ名前を使い、未作成または停止済みの場合は成功し、稼働中の場合だけ停止する構成にします。Docker CLIまたはdaemonへの接続に失敗した場合はエラーを返します。

停止済みコンテナを再利用するため、既存コンテナはイメージを再ビルドしても新しいイメージへ切り替わりません。Dockerfileの変更を反映する場合は、既存コンテナを削除して再作成する扱いを別途決めます。

`IMAGE_NAME`でイメージ名を指定します。例の`example-project-service`はプロジェクト名と役割名を表す仮の値です。各プロジェクトで重複しない名前を決め、既定値または`IMAGE_NAME`に設定します。

Dockerコマンドは実行ユーザーの権限で呼び出し、コンテナ内のプロセスも同じUIDとGIDで実行します。

## docker compose方式

compose定義を使ってコンテナを起動する方式です。

```text
<リポジトリ>/
└── docker/
    ├── compose.yaml
    ├── api/
    │   └── Dockerfile
    └── worker/
        └── Dockerfile
```

`docker/compose.yaml`の例です。

```yaml
name: example-project

services:
  api:
    image: ${API_IMAGE_NAME:-example-project-api}
    build:
      context: ./api
      dockerfile: Dockerfile
  worker:
    image: ${WORKER_IMAGE_NAME:-example-project-worker}
    build:
      context: ./worker
      dockerfile: Dockerfile
```

`name`はDocker Composeのプロジェクト識別子であり、docker-centerのEntry表示名とは別です。例の`example-project`はDockerホスト上で一意になるプロジェクト名に変更します。サービス名はコンテナの役割に合わせます。イメージ名の`example-project-api`と`example-project-worker`はプロジェクト名と役割名を表す仮の値です。各プロジェクトに合う既定値を設定し、必要に応じて`API_IMAGE_NAME`、`WORKER_IMAGE_NAME`で上書きします。各`build.context`には、そのサービスのDockerfileとビルドに必要なファイルだけを含むディレクトリを指定します。
