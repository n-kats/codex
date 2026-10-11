# docker-center対応方法

docker-centerは、Dockerコンテナの登録、状態確認、操作を行うアプリケーションです。

この資料では、リポジトリ内のDocker関連ファイルをdocker-centerで利用できるようにするための実装方法を説明します。

## 起動スクリプト方式

リポジトリ内の起動スクリプトからDocker CLIを呼び出す方式です。単一コンテナは`scripts/launch.sh`、個別に扱う複数コンテナは`scripts/launch_<サービス名>.sh`のようにスクリプトを分けます。同じ起動スクリプトから複数のコンテナを起動する構成もあります。

```text
<リポジトリ>/
├── docker/
│   └── Dockerfile
└── scripts/
    └── launch.sh
```

複数のサービスを個別に扱い、それぞれにDockerfileを使う構成例です。

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

実行時のプロジェクトルートはビルドコンテキストと分け、`/workspace`へマウントします。

docker-centerは起動スクリプトに`CONTAINER_NAME_PREFIX`を環境変数として渡します。コンテナ名は`<CONTAINER_NAME_PREFIX><役割名>`とし、`docker run --name`に指定します。役割名は固定し、同じ接頭辞の中で重複させません。単一コンテナでは`service`、複数のサービスでは`api`や`worker`などを役割名に使います。個別に操作する場合は、サービスごとに起動スクリプトを分けます。

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

docker build -t "$image_name" -f docker/Dockerfile docker
docker run -d --name "$container_name" \
  --user "$(id -u):$(id -g)" -v "$PWD:/workspace" -w /workspace \
  "$image_name" >/dev/null
```

同名のコンテナが稼働中なら、起動スクリプトは何もせず終了します。停止中なら`docker start`で再開し、コンテナがなければイメージをビルドして新規作成します。

停止スクリプトも同じコンテナ名を使います。コンテナが未作成または停止済みなら成功として終了し、稼働中なら停止します。Docker CLIやdaemonでエラーが起きた場合は、スクリプトを失敗させます。

停止済みコンテナを再利用するため、既存コンテナはイメージを再ビルドしても新しいイメージへ切り替わりません。Dockerfileの変更を反映する場合は、既存コンテナを削除して再作成する扱いを別途決めます。

`IMAGE_NAME`でイメージ名を指定します。例の`example-project-service`はプロジェクト名と役割名を組み合わせた仮の値です。実際の名前はプロジェクトごとに決め、既定値または`IMAGE_NAME`に設定します。

Dockerコマンドは実行ユーザーの権限で呼び出し、コンテナ内のプロセスも同じUIDとGIDで実行します。

### Composeを使う起動・停止スクリプト方式

Composeで起動し、停止時にコンテナとComposeプロジェクトのネットワークを削除する構成です。起動と停止は別々のスクリプトで実装します。

```text
<リポジトリ>/
├── docker/
│   ├── compose.yaml
│   ├── api/
│   │   └── Dockerfile
│   └── worker/
│       └── Dockerfile
└── scripts/
    ├── launch.sh
    └── down.sh
```

docker-centerには、操作方式`script`、起動コマンド`bash scripts/launch.sh`、停止コマンド`bash scripts/down.sh`、検出方式`container_name_prefix`を登録します。docker-centerは起動・停止の両方で同じ`CONTAINER_NAME_PREFIX`を渡します。

`docker/compose.yaml`では、各サービスのコンテナ名に同じ接頭辞を付けます。

```yaml
services:
  api:
    container_name: ${CONTAINER_NAME_PREFIX}api
    build:
      context: ./api
      dockerfile: Dockerfile
    working_dir: /workspace
    volumes:
      - ../:/workspace
  worker:
    container_name: ${CONTAINER_NAME_PREFIX}worker
    build:
      context: ./worker
      dockerfile: Dockerfile
    working_dir: /workspace
    volumes:
      - ../:/workspace
```

起動スクリプトでは、接頭辞をComposeプロジェクト名にも使います。

```bash
#!/usr/bin/env bash
set -e
: "${CONTAINER_NAME_PREFIX:?CONTAINER_NAME_PREFIXを設定してください}"
script_dir="$(realpath "$(dirname "$0")")" || exit 1
cd "$(dirname "$script_dir")" || exit 1
docker compose -p "$CONTAINER_NAME_PREFIX" -f docker/compose.yaml up --build --detach --remove-orphans
```

停止スクリプトでは、起動時と同じプロジェクト名とComposeファイルを指定します。Composeが作成したコンテナとネットワークを削除し、名前付きボリュームは保持します。

```bash
#!/usr/bin/env bash
set -e
: "${CONTAINER_NAME_PREFIX:?CONTAINER_NAME_PREFIXを設定してください}"
script_dir="$(realpath "$(dirname "$0")")" || exit 1
cd "$(dirname "$script_dir")" || exit 1
docker compose -p "$CONTAINER_NAME_PREFIX" -f docker/compose.yaml down --remove-orphans
```

`CONTAINER_NAME_PREFIX`には、同じDockerホスト上で重複せず、Composeプロジェクト名として使える値を設定します。

## docker compose方式

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
    working_dir: /workspace
    volumes:
      - ../:/workspace
  worker:
    image: ${WORKER_IMAGE_NAME:-example-project-worker}
    build:
      context: ./worker
      dockerfile: Dockerfile
    working_dir: /workspace
    volumes:
      - ../:/workspace
```

`name`にはDockerホスト上で一意になるComposeプロジェクト名を設定します。これはdocker-centerのEntry表示名とは別です。

サービス名はコンテナの役割に合わせます。イメージ名の既定値にはプロジェクト名とサービス名を含めます。必要に応じて`API_IMAGE_NAME`、`WORKER_IMAGE_NAME`で上書きします。

各`build.context`には、そのサービスのDockerfileとビルドに必要なファイルだけを含むディレクトリを指定します。

## 共通の実装上の注意

`launch.sh`、`down.sh`などのスクリプトは、特別な指示がない限り追加依存を最小限にします。Pythonやjqなどの実行環境・ツールを前提にせず、シェルとDocker CLIを中心に実装します。

## 実装のヒント

### Dockerfileとビルドコンテキスト

同一リポジトリで使うDockerfileが1つなら`docker/Dockerfile`に置き、`docker/`をビルドコンテキストにします。サービスごとに別のDockerfileを使う場合は、`docker/<サービス名>/Dockerfile`に分け、各サービスのディレクトリをビルドコンテキストにします。コンテナや起動スクリプトが複数あっても、Dockerfileをサービスごとに分けるとは限りません。

この資料では、`.dockerignore`で不要ファイルを除外する構成をアンチパターンとします。各ビルドコンテキストにはDockerfileとイメージ作成に必要なファイルだけを置きます。リポジトリルート全体をコンテキストにせず、`.env`、認証情報、秘密鍵などをコンテキストに含めないでください。

アプリケーションソースはDockerfileの`COPY`でイメージに含めず、実行時にプロジェクトルートを`/workspace`へマウントします。

### Pythonの依存環境を起動時に作る場合

ここではPythonを例に説明します。Python本体と依存管理ツールはイメージに含め、アプリケーションのパッケージはイメージ作成時にインストールしません。コンテナ起動時に、ロックファイルなど固定された情報から専用環境へ同期します。

uvと同様に、アプリケーションの実行に使うパッケージ依存を固定情報から同期でき、同期先の環境と取得キャッシュの保存先を指定できるツールにも、この運用を適用します。OSパッケージやDockerビルドキャッシュは対象外です。

#### uvの設定例

uvを使う場合は、DockerイメージにPythonと`uv`コマンドを用意します。

管理対象ごとに、ホスト側の`_cache/<ツール名>_uv_cache`をuvのキャッシュに、`_cache/<ツール名>_uv_venv`を仮想環境に使います。プロジェクトルートを`/workspace`へマウントすると、コンテナ内からはそれぞれ`/workspace/_cache/<ツール名>_uv_cache`、`/workspace/_cache/<ツール名>_uv_venv`として見えます。

この2つのコンテナ内パスを、それぞれ`UV_CACHE_DIR`と`UV_PROJECT_ENVIRONMENT`に設定します。コンテナ内の実行ユーザーが書き込めるようにしてください。これらは実行時の保存先であり、Dockerビルドキャッシュとは別です。

docker composeで起動する場合は、各サービスに`working_dir: /workspace`を設定します。この資料の例のようにComposeファイルが`docker/compose.yaml`にある場合は、`volumes`に`../:/workspace`を指定してプロジェクトルートをマウントします。`UV_CACHE_DIR`と`UV_PROJECT_ENVIRONMENT`には、起動スクリプトの場合と同じコンテナ内パスを設定します。

コンテナ内で実行する起動処理の例です。`example-project-service`は管理対象ごとの名前です。実際の名前とアプリケーションの起動コマンドは各プロジェクトに合わせます。

```sh
target_name="example-project-service"
export UV_CACHE_DIR="/workspace/_cache/${target_name}_uv_cache"
export UV_PROJECT_ENVIRONMENT="/workspace/_cache/${target_name}_uv_venv"
uv sync --locked
exec uv run --no-sync python -m example_project
```
