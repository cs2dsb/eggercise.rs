#!/usr/bin/env bash

# Exit when any command fails
set -o errexit

# Exit when an undeclared variable is used
set -o nounset

# Exit when a piped command returns a non-zero exit code
set -o pipefail

readonly repo_dir="$( cd $(dirname ${BASH_SOURCE}); pwd )"
cd "${repo_dir}"

readonly release_dir="${repo_dir}/releases";
readonly db_dir="${repo_dir}/database";
readonly keys_dir="${repo_dir}/keys";

readonly docker_image="ghcr.io/cs2dsb/eggercise.rs:latest";
readonly app_user=web-apps;
readonly server_name=server;
readonly service_file=eggercise_rs.service;
readonly webhook_service_file=eggercise_webhook.service;
readonly hook_config_file=hook_config.json;


# Set to false to prevent the final restart to do a sanity check
readonly restart_services=true

function error_exit() {
    echo "ERROR: ${1:-}" >&2;
    exit 1;
}

cd "${repo_dir}"
mkdir -p "${db_dir}"

sed \
	-e "s|\${REDEPLOY_COMMAND}|${repo_dir}/redeploy.sh|g" \
	${hook_config_file}.template \
	| sudo -u ${app_user} tee ${hook_config_file}

sed \
	-e "s|\${APP_USER}|${app_user}|g" \
	-e "s|\${WORKING_DIRECTORY}|${release_dir}|g" \
	-e "s|\${EXEC_START}|${release_dir}/latest_${server_name}|g" \
	${service_file}.template \
	| sudo -u ${app_user} tee ${service_file}

sed \
	-e "s|\${WEBHOOK_PATH}|${repo_dir}/webhook|g" \
	-e "s|\${HOOK_CONFIG}|${repo_dir}/hook_config.json|g" \
	${webhook_service_file}.template \
	| sudo -u ${app_user} tee ${webhook_service_file}

sudo systemctl daemon-reload
sudo systemctl enable "${repo_dir}/${service_file}"
sudo systemctl enable "${repo_dir}/${webhook_service_file}"

# Fix the permissions
sudo chown -R ${app_user}:${app_user} ${repo_dir}

# Pull the new image
sudo docker pull "$docker_image"

# Stop and remove anything descended from it
sudo systemctl stop $service_file
sudo docker stop eggercise.rs
sudo docker rm eggercise.rs
readonly containers=$(sudo docker ps -a -q  \
	--filter ancestor="$docker_image" \
	--format="{{.ID}}");
if [[ "$containers" != "" ]]; then
	sudo docker stop $containers
	sudo docker rm $containers
fi

# Kick off the new instance
sudo docker run -d \
	--name=eggercise.rs \
	-e ASSETS_DIR=/opt/server/assets \
	-e SQLITE_CONNECTION_STRING=/opt/server/database/egg.sqlite \
	-v "${db_dir}":/opt/server/database \
	-v "${keys_dir}":/opt/server/keys \
	-v "${repo_dir}/.env":/server/.env \
	-e WEBAUTHN_ORIGIN=https://egg.ileet.co.uk \
	-e WEBAUTHN_ID=egg.ileet.co.uk \
	-e CORS_ORIGIN=https://egg.ileet.co.uk \
	-e PRIVATE_KEY_PATH=/opt/server/keys/egg_key.pem \
	-e PUBLIC_KEY_PATH=/opt/server/keys/egg_key.pub.pem \
	-p 9090:9090 \
	"$docker_image"

# Prune any unused images
sudo docker image prune --force --all

if [ "$restart_services" == "true" ]; then
	# Restart the services
	echo "Restarting services"
	sudo systemctl restart ${service_file}
	sudo systemctl restart ${webhook_service_file}
fi
