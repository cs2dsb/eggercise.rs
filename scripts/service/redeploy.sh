#!/usr/bin/env bash

# Exit when any command fails
set -o errexit

# Exit when an undeclared variable is used
set -o nounset

# Exit when a piped command returns a non-zero exit code
set -o pipefail

readonly repo_dir="$( cd $(dirname ${BASH_SOURCE}); pwd )"
cd "${repo_dir}"

readonly release_dir="${repo_dir}/releases"

readonly app_user=web-apps;
readonly server_name=server;
readonly service_file=eggercise_rs.service;
readonly webhook_service_file=eggercise_webhook.service;
readonly hook_config_file=hook_config.json

readonly release_url_base=https://github.com/cs2dsb/eggercise.rs/releases/tag;
readonly api_url="https://api.github.com/repos/cs2dsb/eggercise.rs/releases";

# Set to false to prevent the final restart to do a sanity check
readonly restart_services=false
# Set to false to prevent linking download to latest, also prevents service restart
readonly link_latest=true

# Set to the bins you want to extract
readonly bins=(server)

# Get the tag passed from the webhook
readonly tag=${1:-};

function error_exit() {
    echo "ERROR: ${1:-}" >&2;
    exit 1;
}

[[ "${tag}" == "" ]] && error_exit "First argument should be release tag name"

sudo -u ${app_user} mkdir -p "${release_dir}"
cd "${release_dir}"

# Fetch the releases page. This is sorted in reverse order so we don't have to worry about pagenation
# Saved to a file as the rate limit is very low for unauthenticated requests
sudo -u ${app_user} curl -sL ${api_url} -o releases.json

for bin in ${bins[@]}; do
	# The pattern for the artifact we want
	dl_file_pattern="${bin}_.*_linux.tar.gz";

	# The url to for the artifact
	dl_url=`cat releases.json | jq -r ".[] | select(.tag_name == \"${1}\") | .assets[] | select(.name|test(\"${dl_file_pattern}\")) | .browser_download_url"`;

	# Check we found it
	[[ "${dl_url}" == "" ]] && error_exit "Failed to find download url for ${dl_file_pattern} in release with tag \"${tag}\". Check ${gb_deploy_dir}/releases.json"

	# The local download file name
	gz_file="${bin}_${tag}_linux.tar.gz";

	# Download it
	sudo -u ${app_user} curl -sL ${dl_url} -o "${gz_file}";

	# Extract it
	sudo -u ${app_user} tar -xf "${gz_file}";

	# Delete the archive
	sudo -u ${app_user} rm "${gz_file}";

	# Rename the binary to include the tag
	tagged_name="${bin}_${tag}";
	sudo -u ${app_user} mv "${bin}" "${tagged_name}";

	if [ "${link_latest}" == "true" ]; then
		# The link name for the latest version
		ln_name="latest_${bin}";

		# Delete the link
		sudo -u ${app_user} rm -f "${ln_name}"

		# Recreate it
		sudo -u ${app_user} ln -s "${tagged_name}" "${ln_name}";
	fi
done

cd "${repo_dir}"

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

if [ "$restart_services" == "true" ] && [ "$link_latest" == "true" ]; then
	# Restart the services
	echo "Restarting services"
	sudo systemctl restart ${service_file}
	sudo systemctl restart ${webhook_service_file}
fi
