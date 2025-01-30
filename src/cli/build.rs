use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
    path::PathBuf,
};

use clap::{ArgAction, Args};
use color_eyre::eyre::{bail, ensure, eyre, OptionExt, WrapErr};
use compose_spec::{
    service::{
        self,
        build::{Cache, CacheType, Context, Dockerfile},
        ByteValue, Limit, Ulimit,
    },
    ShortOrLong,
};
use serde::Serialize;
use smart_default::SmartDefault;

use crate::{
    quadlet::{
        self,
        build::Secret,
        container::{Device, DnsEntry, PullPolicy},
    },
    serde::skip_true,
};

use super::image_to_name;

/// [`Args`] for `podman build`.
#[allow(clippy::doc_markdown)]
#[derive(Args, Debug, SmartDefault, Clone, PartialEq, Eq)]
#[group(skip)]
pub struct Build {
    /// Add an image annotation (e.g. annotation=value) to the image metadata.
    ///
    /// Converts to "Annotation=ANNOTATION=VALUE".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "ANNOTATION=VALUE")]
    annotation: Vec<String>,

    /// Set the architecture of the image to be built, instead of using the build host's.
    ///
    /// Converts to "Arch=ARCH".
    #[arg(long)]
    arch: Option<String>,

    /// Path of the authentication file.
    ///
    /// Converts to "AuthFile=PATH".
    #[arg(long, value_name = "PATH")]
    authfile: Option<PathBuf>,

    /// Set custom DNS servers.
    ///
    /// Converts to "DNS=IP_ADDRESS".
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "IP_ADDRESS")]
    // TODO: use `Dns` directly if clap ever supports custom collections (https://github.com/clap-rs/clap/issues/3114).
    dns: Vec<DnsEntry>,

    /// Set custom DNS options to be used during the build.
    ///
    /// Converts to "DNSOption=OPTION".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "OPTION")]
    dns_option: Vec<String>,

    /// Set custom DNS search domains to be used during the build.
    ///
    /// Converts to "DNSSearch=DOMAIN".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "DOMAIN")]
    dns_search: Vec<String>,

    /// Add a value (e.g. env=value) to the built image.
    ///
    /// Converts to "Environment=ENV[=VALUE]".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "ENV[=VALUE]")]
    env: Vec<String>,

    /// Specifies a Containerfile which contains instructions for building the image.
    ///
    /// Converts to "File=CONTAINERFILE".
    ///
    /// Either this option or the `context` argument is required.
    #[arg(
        short,
        long,
        required_unless_present = "context",
        value_name = "CONTAINERFILE"
    )]
    file: Option<Context>,

    /// Always remove intermediate containers after a build, even if the build fails.
    ///
    /// Converts to "ForceRM=FORCE_RM".
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[default = true]
    force_rm: bool,

    /// Assign additional groups to the primary user running within the container process.
    ///
    /// Converts to "GroupAdd=GROUP".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "GROUP | keep-groups")]
    group_add: Vec<String>,

    /// The name assigned to the resulting image if the build process completes successfully.
    ///
    /// Converts to "ImageTag=IMAGE_NAME".
    ///
    /// This option is required by Quadlet.
    #[arg(short, long, value_name = "IMAGE_NAME")]
    tag: String,

    /// Add an image label (e.g. label=value) to the image metadata.
    ///
    /// Converts to "Label=LABEL=VALUE".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "LABEL=VALUE")]
    label: Vec<String>,

    /// Sets the configuration for network namespaces when handling `RUN` instructions.
    ///
    /// Converts to "Network=MODE".
    ///
    /// Can be specified multiple times.
    #[arg(long, visible_alias = "net", value_name = "MODE")]
    network: Vec<String>,

    /// Pull image policy.
    ///
    /// Converts to "Pull=POLICY".
    #[arg(long, value_name = "POLICY")]
    pull: Option<PullPolicy>,

    /// Pass secret information in a safe way to the build container.
    ///
    /// Converts to "Secret=id=ID,src=PATH".
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "id=ID,src=PATH")]
    secret: Vec<Secret>,

    /// Set the target build stage to build.
    ///
    /// Converts to "Target=STAGE_NAME".
    #[arg(long, value_name = "STAGE_NAME")]
    target: Option<String>,

    /// Require HTTPS and verify certificates when contacting registries.
    ///
    /// Converts to "TLSVerify=TLS_VERIFY".
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    tls_verify: Option<bool>,

    /// Set the architecture variant of the image to be built, instead of using the build host's.
    #[arg(long)]
    variant: Option<String>,

    /// Mount a host directory into containers when executing `RUN` instructions during the build.
    ///
    /// The host and container directory paths must be absolute.
    ///
    /// Converts to "Volume=[HOST_DIR:]CONTAINER_DIR[:OPTIONS]".
    ///
    /// Can be specified multiple times.
    #[arg(short, long, value_name = "[HOST_DIR:]CONTAINER_DIR[:OPTIONS]")]
    volume: Vec<String>,

    /// Arguments that do not have a more specific Quadlet option.
    ///
    /// Converts to "PodmanArgs=ARGS".
    #[command(flatten)]
    podman_args: PodmanArgs,

    /// Build context directory.
    ///
    /// Converts to "SetWorkingDirectory=CONTEXT".
    ///
    /// Either this argument or the `--file` option is required.
    #[arg(required_unless_present = "file")]
    context: Option<Context>,
}

impl Build {
    /// The name (without extension) of the generated Quadlet file.
    pub fn name(&self) -> &str {
        image_to_name(&self.tag)
    }
}

impl From<Build> for quadlet::Build {
    fn from(
        Build {
            annotation,
            arch,
            authfile,
            dns,
            dns_option,
            dns_search,
            env,
            file,
            force_rm,
            group_add,
            tag,
            label,
            network,
            pull,
            secret,
            target,
            tls_verify,
            variant,
            volume,
            podman_args,
            context,
        }: Build,
    ) -> Self {
        let podman_args = podman_args.to_string();

        Self {
            annotation,
            arch,
            auth_file: authfile,
            dns: dns.into(),
            dns_option,
            dns_search,
            environment: env,
            file,
            force_rm,
            group_add,
            image_tag: tag,
            label,
            network,
            podman_args: (!podman_args.is_empty()).then_some(podman_args),
            pull,
            secret,
            set_working_directory: context,
            target,
            tls_verify,
            variant,
            volume,
        }
    }
}

impl From<Build> for quadlet::Resource {
    fn from(value: Build) -> Self {
        quadlet::Build::from(value).into()
    }
}

impl TryFrom<service::Build> for Build {
    type Error = color_eyre::Report;

    fn try_from(
        service::Build {
            context,
            dockerfile,
            args,
            ssh,
            cache_from,
            cache_to,
            additional_contexts,
            entitlements,
            extra_hosts,
            isolation,
            privileged,
            labels,
            no_cache,
            pull,
            network,
            shm_size,
            target,
            secrets,
            tags,
            ulimits,
            platforms,
            extensions,
        }: service::Build,
    ) -> Result<Self, Self::Error> {
        ensure!(entitlements.is_empty(), "`entitlements` are not supported");
        ensure!(!privileged, "`privileged` is not supported");
        ensure!(secrets.is_empty(), "`secrets` are not supported");
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        let file = dockerfile
            .map(|dockerfile| match dockerfile {
                Dockerfile::File(file) => Ok(file.into()),
                Dockerfile::Inline(_) => Err(eyre!("`dockerfile_inline` is not supported")),
            })
            .transpose()?;

        ensure!(
            context.is_some() || file.is_some(),
            "`context` or `dockerfile` is required"
        );

        let podman_args = PodmanArgs {
            add_host: extra_hosts
                .into_iter()
                .map(|(hostname, ip)| format!("{hostname}:{ip}"))
                .collect(),
            build_arg: args.into_list().into_iter().collect(),
            build_context: additional_contexts
                .iter()
                .map(|(id, context)| format!("{id}={context}"))
                .collect(),
            cache_from: cache_from
                .into_iter()
                .map(cache_try_into_image)
                .collect::<Result<_, _>>()
                .wrap_err("error converting `cache_from`")?,
            cache_to: cache_to
                .into_iter()
                .map(cache_try_into_image)
                .collect::<Result<_, _>>()
                .wrap_err("error converting `cache_to`")?,
            isolation,
            no_cache,
            platform: platforms.iter().map(ToString::to_string).collect(),
            shm_size,
            ssh: ssh.iter().map(ToString::to_string).collect(),
            ulimit: ulimits
                .into_iter()
                .map(|(resource, ulimit)| match ulimit {
                    ShortOrLong::Short(limit) => Ok(format!("{resource}={limit}")),
                    ShortOrLong::Long(Ulimit {
                        soft,
                        hard,
                        extensions,
                    }) => {
                        ensure!(
                            extensions.is_empty(),
                            "compose extensions are not supported"
                        );
                        Ok(format!("{resource}={soft}:{hard}"))
                    }
                })
                .collect::<Result<_, _>>()
                .wrap_err("error converting `ulimits`")?,
            ..PodmanArgs::default()
        };

        let mut tags = tags.into_iter();
        let tag = tags
            .next()
            .map(|t| t.into_inner())
            .unwrap_or_else(|| "latest".to_string());
        ensure!(
            tags.next().is_none(),
            "Quadlet only supports setting a single tag"
        );

        Ok(Self {
            file,
            tag,
            label: labels.into_list().into_iter().collect(),
            network: network.map(Into::into).into_iter().collect(),
            pull: pull.then_some(PullPolicy::Always),
            target,
            podman_args,
            context,
            ..Self::default()
        })
    }
}

/// Attempt to convert [`Cache`] into an image name.
///
/// # Errors
///
/// Returns an error if the cache type is not [`Registry`](CacheType::Registry) or any other cache
/// options are set.
fn cache_try_into_image(
    Cache {
        cache_type,
        options,
    }: Cache,
) -> color_eyre::Result<String> {
    let image = match cache_type {
        CacheType::Registry(image) => image.into_inner(),
        CacheType::Other(_) => bail!("only the `registry` cache type is supported"),
    };
    ensure!(options.is_empty(), "cache options are not supported");
    Ok(image)
}

/// [`Args`] for `podman build` (i.e. [`Build`]) that convert into `PodmanArgs=ARGS`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Serialize, Debug, SmartDefault, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct PodmanArgs {
    /// Add a custom host-to-IP mapping.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "HOST:IP")]
    add_host: Vec<String>,

    /// Build for all platforms which are available for the base image.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    all_platforms: bool,

    /// Specifies a build argument and its value.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "ARG=VALUE")]
    build_arg: Vec<String>,

    /// Specifies a file containing lines of build arguments of the form `arg=value`.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "PATH")]
    build_arg_file: Vec<PathBuf>,

    /// Specify an additional build context using its short name and its location.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "NAME=VALUE")]
    build_context: Vec<String>,

    /// Repository to utilize as a potential cache source.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "IMAGE")]
    cache_from: Vec<String>,

    /// Set this flag to specify a remote repository that is used to store cache images.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "IMAGE")]
    cache_to: Vec<String>,

    /// Limit the use of cached images to only those with created less than `DURATION` ago.
    #[arg(long, value_name = "DURATION")]
    cache_ttl: Option<String>,

    /// When executing a `RUN` instruction, add the specified capability to its capability set.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "CAPABILITY")]
    cap_add: Vec<String>,

    /// When executing a `RUN` instruction, remove the specified capability from its capability set.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "CAPABILITY")]
    cap_drop: Vec<String>,

    /// Use certificates at path (*.crt, *.cert, *.key) to connect to the registry.
    #[arg(long, value_name = "PATH")]
    cert_dir: Option<PathBuf>,

    /// Path to cgroups under which the cgroup for the container is created.
    #[arg(long, value_name = "PATH")]
    cgroup_parent: Option<PathBuf>,

    /// Sets the configuration for cgroup namespaces when handling `RUN` instructions.
    #[arg(long, value_name = "HOW")]
    cgroupns: Option<String>,

    /// Preserve the contents of `VOLUME`s during `RUN` instructions.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    compat_volumes: bool,

    /// Ignored by Podman.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    compress: bool,

    /// Set additional flags to pass to the C Preprocessor cpp(1).
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "FLAGS")]
    cpp_flag: Vec<String>,

    /// Set the CPU period for the CFS (Completely Fair Scheduler), in microseconds.
    #[arg(long, value_name = "LIMIT")]
    cpu_period: Option<u128>,

    /// Limit the CPU CFS (Completely Fair Scheduler) quota, in microseconds.
    #[arg(long, value_name = "LIMIT")]
    cpu_quota: Option<u128>,

    /// CPU shares (relative weight).
    #[arg(short, long, value_name = "SHARES")]
    cpu_shares: Option<u64>,

    /// CPUs in which to allow execution.
    #[arg(long, value_name = "NUMBER")]
    cpuset_cpus: Option<String>,

    /// Memory nodes (MEMs) in which to allow execution.
    #[arg(long, value_name = "NODES")]
    cpuset_mems: Option<String>,

    /// The username and/or password to use to authenticate with the registry, if required.
    #[arg(long, value_name = "[USERNAME][:PASSWORD]")]
    creds: Option<String>,

    /// Confidential workload options.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "OPTIONS")]
    cw: Vec<String>,

    /// The key and optional passphrase to be used for decryption of images.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "KEY[:PASSPHRASE]")]
    decryption_key: Vec<String>,

    /// Add a host device to the build container.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "HOST_DEVICE[:CONTAINER_DEVICE][:PERMISSIONS]")]
    device: Vec<Device>,

    /// Don't compress filesystem layers by default.
    #[arg(short = 'D', long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    disable_compression: bool,

    /// This is a Docker specific option and is a NOOP.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    disable_content_trust: bool,

    /// Control the format for the built image's manifest and configuration data.
    #[arg(long)]
    format: Option<String>,

    /// Overrides the first FROM instruction within the Containerfile.
    #[arg(long)]
    from: Option<String>,

    /// Each `*.json` file in the path configures a hook for Buildah build containers.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "PATH")]
    hooks_dir: Vec<PathBuf>,

    /// Pass proxy environment variables into the build container.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    http_proxy: bool,

    /// Adds default identity label `io.buildah.version` if set.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    identity_label: bool,

    /// Path to an alternative `.containerignore` file.
    #[arg(long, value_name = "PATH")]
    ignorefile: Option<PathBuf>,

    /// Write the built image's ID to a file.
    #[arg(long, value_name = "IMAGE_ID_FILE")]
    iidfile: Option<PathBuf>,

    /// Sets the configuration for IPC namespaces when handling `RUN` instructions.
    #[arg(long, value_name = "HOW")]
    ipc: Option<String>,

    /// Controls what type of isolation is used for running processes as part of RUN instructions.
    #[arg(long, value_name = "TYPE")]
    isolation: Option<String>,

    /// Run up to N concurrent stages in parallel.
    #[arg(long, value_name = "NUMBER")]
    jobs: Option<u64>,

    /// Add an intermediate image label (e.g. label=value) to the intermediate image metadata.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "LABEL[=VALUE]")]
    layer_label: Vec<String>,

    /// Cache intermediate images during the build process.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    layers: bool,

    /// Log output to the specified file instead of standard output and standard error.
    #[arg(long, value_name = "FILENAME")]
    logfile: Option<PathBuf>,

    /// Split the log file into different files for each platform.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    logsplit: bool,

    /// Name of the manifest list to which the image is added.
    #[arg(long)]
    manifest: Option<String>,

    /// Memory limit.
    #[arg(short, long, value_name = "NUMBER[UNIT]")]
    memory: Option<ByteValue>,

    /// A limit value equal to memory plus swap.
    #[arg(long, allow_negative_numbers = true, value_name = "NUMBER[UNIT]")]
    memory_swap: Option<Limit<ByteValue>>,

    /// Do not use existing cached images for the container build.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_cache: bool,

    /// Do not create the `/etc/hostname` file in the container for `RUN` instructions.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_hostname: bool,

    /// Do not create `/etc/hosts` for the container.
    #[arg(long, conflicts_with = "add_host")]
    #[serde(skip_serializing_if = "Not::not")]
    no_hosts: bool,

    /// Omit build history information in the built image.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    omit_history: bool,

    /// Set the operating system of the image to be built, instead of using the build host's.
    #[arg(long)]
    os: Option<String>,

    /// Set the name of a required operating system feature for the image which is built.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "FEATURE")]
    os_feature: Vec<String>,

    /// Set the exact required operating system version for the image which is built.
    #[arg(long, value_name = "VERSION")]
    os_version: Option<String>,

    /// Output destination.
    #[arg(short, long, value_name = "OUTPUT_OPTS")]
    output: Option<String>,

    /// Sets the configuration for PID namespaces when handling `RUN` instructions.
    #[arg(long)]
    pid: Option<String>,

    /// Set the os/arch of the built image, instead of using the build host's.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "OS/ARCH[/VARIANT][,...]")]
    platform: Vec<String>,

    /// Suppress output messages.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    quiet: bool,

    /// Number of times to retry pulling images from the registry in case of failure.
    #[arg(long, value_name = "ATTEMPTS")]
    retry: Option<u64>,

    /// Duration of delay between retry attempts.
    #[arg(long, value_name = "DURATION")]
    retry_delay: Option<String>,

    /// Remove intermediate containers after a successful build.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    rm: bool,

    /// The path to an alternate OCI-compatible runtime, which is used by `RUN` instructions.
    #[arg(long, value_name = "PATH")]
    runtime: Option<PathBuf>,

    /// Adds global flags for the container runtime.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "FLAG")]
    runtime_flag: Vec<String>,

    /// Generate SBOMs (Software Bills Of Materials) for the output image.
    #[arg(long, value_name = "PRESET")]
    sbom: Option<String>,

    /// When generating SBOMs, store the generated SBOM in the specified path in the output image.
    #[arg(long, value_name = "PATH")]
    sbom_image_output: Option<PathBuf>,

    /// When generating SBOMs, scan them for PURLs, and save a list to the path in the output image.
    #[arg(long, value_name = "PATH")]
    sbom_image_purl_output: Option<PathBuf>,

    /// If more than one `--sbom-scanner-command` is set, use the given method to merge the output.
    #[arg(long, value_name = "METHOD")]
    sbom_merge_strategy: Option<String>,

    /// When generating SBOMs, store the generated SBOM in the named file on the local filesystem.
    #[arg(long, value_name = "FILE")]
    sbom_output: Option<PathBuf>,

    /// When generating SBOMs, scan them for PURLs, and save a list to the named file.
    #[arg(long, value_name = "FILE")]
    sbom_purl_output: Option<PathBuf>,

    /// Generate SBOMs by running the specified command from the scanner image.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "COMMAND")]
    sbom_scanner_command: Vec<String>,

    /// Generate SBOMs using the specified scanner image.
    #[arg(long, value_name = "IMAGE")]
    sbom_scanner_image: Option<String>,

    /// Security options.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "OPTION")]
    security_opt: Vec<String>,

    /// Size of `/dev/shm`.
    #[arg(long, value_name = "NUMBER[UNIT]")]
    shm_size: Option<ByteValue>,

    /// Sign the image using a GPG key with the specified fingerprint.
    #[arg(long, value_name = "FINGERPRINT")]
    sign_by: Option<String>,

    /// Skip stages in multi-stage builds which don't affect the target stage.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    skip_unused_stages: bool,

    /// Squash all of the image's new layers into a single new layer.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    squash: bool,

    /// Squash all of the image's layers, including the base image's, into a single new layer.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    squash_all: bool,

    /// SSH agent socket or keys to expose to the build.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "default | ID[=SOCKET] | KEY[,...]")]
    ssh: Vec<String>,

    /// Pass stdin into the `RUN` containers.
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    stdin: bool,

    /// Set the create timestamp to seconds since epoch to allow for deterministic builds.
    #[arg(long, value_name = "SECONDS")]
    timestamp: Option<u64>,

    /// Specifies resource limits to apply to processes launched when processing `RUN` instructions.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "TYPE=SOFT_LIMIT[:HARD_LIMIT]")]
    ulimit: Vec<String>,

    /// Unset environment variables from the final image.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "ENV")]
    unsetenv: Vec<String>,

    /// Unset the image label, causing the label not to be inherited from the base image.
    ///
    /// Can be specified multiple times.
    #[arg(long, value_name = "LABEL")]
    unsetlabel: Vec<String>,

    /// Sets the configuration for user namespaces when handling `RUN` instructions.
    #[arg(long, value_name = "HOW")]
    userns: Option<String>,

    /// Directly specifies a GID mapping to be used.
    #[arg(long, value_name = "MAPPING")]
    userns_gid_map: Option<String>,

    /// Specifies that a GID mapping to be used can be found in entries of the `/etc/subgid` file.
    #[arg(long, value_name = "GROUP")]
    userns_gid_map_group: Option<String>,

    /// Directly specifies a UID mapping to be used.
    #[arg(long, value_name = "MAPPING")]
    userns_uid_map: Option<String>,

    /// Specifies that a UID mapping to be used can be found in entries of the `/etc/subuid` file.
    #[arg(long, value_name = "USER")]
    userns_uid_map_user: Option<String>,

    /// Sets the configuration for UTS namespaces when handling `RUN` instructions.
    #[arg(long, value_name = "HOW")]
    uts: Option<String>,
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn podman_args_default_display_empty() {
        let args = PodmanArgs::default();
        assert!(args.to_string().is_empty());
    }
}
