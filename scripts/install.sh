#!/bin/sh
# Bridget user installer. O(archive size); installs and verifies a user service.
set -eu
umask 077

fail() { printf 'Bridget: %s\n' "$*" >&2; exit 1; }
say() { printf 'Bridget: %s\n' "$*" >&2; }
need() { command -v "$1" >/dev/null 2>&1 || fail "Missing prerequisite: $1"; }
download() {
    curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
        --connect-timeout 15 --max-time 300 --retry 2 -o "$2" "$1"
}
digest() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    else
        shasum -a 256 "$1" | cut -d ' ' -f 1
    fi
}
healthy() {
    BRIDGET_HOME="$daemon_root" BRIDGET_SOCKET="$daemon_socket" \
        "$1" agents --json --global > "$work/health.json" 2> "$work/health-error"
}
xml_text() { printf '%s' "$1" | sed 's/\&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }
systemd_quote() { printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/%/%%/g')"; }
shell_literal() { printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"; }

# Closed provider selection. Accounts and models are not installation inputs.
prepare_providers() {
    if [ "$providers" = ask ]; then
        providers=none
        if [ -t 2 ] && [ -r /dev/tty ] && [ -w /dev/tty ]; then
            providers=
            for provider in claude codex gemini cursor; do
                case "$provider" in cursor) program=cursor-agent ;; *) program=$provider ;; esac
                if command -v "$program" >/dev/null 2>&1; then
                    say "$program is already installed; preserved."
                    continue
                fi
                printf 'Bridget: Install %s from its official publisher for this user? [y/N] ' "$provider" >/dev/tty
                answer=
                read -r answer </dev/tty || answer=
                case "$answer" in y|Y|yes|oui) providers="${providers:+$providers,}$provider" ;; esac
            done
            providers=${providers:-none}
        else
            say 'No provider selected: no provider program will be installed. Use --providers claude,codex,gemini,cursor, or --providers none.'
        fi
    fi
    [ "$providers" != none ] || return 0
    # All names were validated before any download or installation.
    selected_providers=$providers
    old_provider_ifs=$IFS
    IFS=,
    for provider in $selected_providers; do
        IFS=$old_provider_ifs
        case "$provider" in cursor) program=cursor-agent ;; *) program=$provider ;; esac
        if command -v "$program" >/dev/null 2>&1; then
            say "$program is already installed; no update or account change."
        else
            say "Installing $provider for this user from its official publisher."
            case "$provider" in
                claude)
                    need bash
                    download https://claude.ai/install.sh "$work/provider-install.sh"
                    bash "$work/provider-install.sh" </dev/null || fail 'Claude Code installation failed; existing Bridget services were preserved.' ;;
                codex)
                    download https://chatgpt.com/codex/install.sh "$work/provider-install.sh"
                    sh "$work/provider-install.sh" </dev/null || fail 'Codex installation failed; existing Bridget services were preserved.' ;;
                cursor)
                    need bash
                    download https://cursor.com/install "$work/provider-install.sh"
                    bash "$work/provider-install.sh" </dev/null || fail 'Cursor CLI installation failed; existing Bridget services were preserved.' ;;
                gemini)
                    need node
                    need npm
                    node -e 'process.exit(Number(process.versions.node.split(".")[0]) >= 20 ? 0 : 1)' || fail 'Gemini CLI needs Node.js 20 or later. Update Node.js explicitly, then repeat --providers gemini. Existing Bridget services were preserved.'
                    npm install --global --prefix "$HOME/.local" @google/gemini-cli </dev/null || fail 'Gemini CLI installation failed; existing Bridget services were preserved.' ;;
            esac
            command -v "$program" >/dev/null 2>&1 || fail "$program was not found after installation. Inspect the official installer result and PATH; no model access is confirmed."
        fi
        case "$provider" in
            claude) login_hint='claude auth login' ;;
            codex) login_hint='codex login' ;;
            gemini) login_hint='gemini' ;;
            cursor) login_hint='cursor-agent login' ;;
        esac
        say "$provider program present. Authentication and model access are NOT checked. Personal sign-in, if needed: $login_hint"
        IFS=,
    done
    IFS=$old_provider_ifs
    say 'Program installation does not declare a remote fleet route or guarantee that parent permissions can be translated. Use the native catalogue and the admission result for the requested project and machine.'
}

prepare_service() {
    service_created=false
    daemon_root=${BRIDGET_HOME:-$HOME/.cache/bridget-core}
    daemon_root=${daemon_root%/}
    daemon_socket=${BRIDGET_SOCKET:-$daemon_root/bridget.sock}
    case "$daemon_root" in /*) ;; *) fail 'BRIDGET_HOME must be absolute.' ;; esac
    case "$daemon_root$daemon_socket" in *'
'*) fail 'Service paths must not contain newlines.' ;; esac
    [ "${daemon_socket%/*}" = "$daemon_root" ] || fail 'BRIDGET_SOCKET must be directly inside BRIDGET_HOME.'
    [ "$(printf '%s' "$daemon_socket" | LC_ALL=C wc -c | tr -d ' ')" -lt 104 ] || fail 'Bridget state path is too long for its socket. Choose a shorter BRIDGET_HOME.'
    if healthy "$candidate"; then
        service_kind=existing
        return
    fi
    # A dead tunnel or uncertain socket must never become a second authority.
    [ ! -e "$daemon_root/federation.env" ] && [ ! -L "$daemon_root/federation.env" ] || fail 'An existing federation is unavailable. Restore its connection; no local daemon was started.'
    [ ! -e "$daemon_socket" ] && [ ! -L "$daemon_socket" ] || fail 'The existing Bridget socket could not be verified with this version. Existing services were preserved; no second daemon was started.'
    uid=$(id -u)
    service_path=$bindir:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
    service_kind=$platform
    if [ "$platform" = darwin ]; then
        need launchctl
        launchctl print "gui/$uid" >/dev/null 2>&1 || fail 'macOS has no graphical login session for this user. Log in at the Mac, then rerun this installer; no service was started.'
        unitdir=$HOME/Library/LaunchAgents
        unit=$unitdir/app.cartae.bridget.plist
        legacy=$unitdir/com.bridget.daemon.plist
        if launchctl print "gui/$uid/app.cartae.bridget" >/dev/null 2>&1; then
            [ -f "$unit" ] && grep -Fq 'Bridget public installer service v1' "$unit" || fail 'The Bridget service label belongs to another installation.'
        fi
        cat > "$work/service" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<!-- Bridget public installer service v1 -->
<key>Label</key><string>app.cartae.bridget</string>
<key>ProgramArguments</key><array><string>$(xml_text "$bindir/bridget")</string><string>daemon</string></array>
<key>WorkingDirectory</key><string>$(xml_text "$HOME")</string>
<key>EnvironmentVariables</key><dict>
<key>HOME</key><string>$(xml_text "$HOME")</string>
<key>PATH</key><string>$(xml_text "$service_path")</string>
<key>BRIDGET_HOME</key><string>$(xml_text "$daemon_root")</string>
<key>BRIDGET_SOCKET</key><string>$(xml_text "$daemon_socket")</string>
</dict>
<key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
<key>Umask</key><integer>63</integer>
<key>ThrottleInterval</key><integer>5</integer>
<key>ExitTimeOut</key><integer>90</integer>
<key>StandardOutPath</key><string>$(xml_text "$state/service-stdout.log")</string>
<key>StandardErrorPath</key><string>$(xml_text "$state/service-stderr.log")</string>
</dict></plist>
EOF
        need plutil
        plutil -lint "$work/service" >/dev/null || fail 'Invalid macOS service configuration.'
    else
        need systemctl
        systemctl --user show-environment >/dev/null 2>&1 || fail 'The systemd user manager is unavailable. Use a normal SSH/login session with systemd user services, then rerun.'
        need loginctl
        linger=$(loginctl show-user "$uid" -p Linger --value) || fail 'Cannot check Linux background startup.'
        if [ "$linger" != yes ]; then
            loginctl --no-ask-password enable-linger "$uid" || fail 'Automatic startup needs permission to keep your user service running after logout. Ask the administrator to enable lingering for your account, then rerun.'
            [ "$(loginctl show-user "$uid" -p Linger --value)" = yes ] || fail 'Linux background startup was not enabled.'
        fi
        config_home=${XDG_CONFIG_HOME:-$HOME/.config}
        case "$config_home" in /*) ;; *) fail 'XDG_CONFIG_HOME must be absolute.' ;; esac
        unitdir=$config_home/systemd/user
        unit=$unitdir/app.cartae.bridget.service
        legacy=$unitdir/bridget-daemon.service
        loaded=$(systemctl --user show app.cartae.bridget.service -p LoadState --value 2>/dev/null || :)
        case "$loaded" in
            not-found) ;;
            loaded)
                fragment=$(systemctl --user show app.cartae.bridget.service -p FragmentPath --value)
                [ "$fragment" = "$unit" ] && [ -f "$unit" ] && grep -Fq 'Bridget public installer service v1' "$unit" || fail 'The running service belongs to another Bridget installation.' ;;
            *) fail 'Cannot establish ownership of the existing Bridget service. Nothing was started.' ;;
        esac
        exec_path=$(systemd_quote "$bindir/bridget" | sed 's/\$/$$/g')
        cat > "$work/service" <<EOF
# Bridget public installer service v1
[Unit]
Description=Bridget communication service
StartLimitIntervalSec=300
StartLimitBurst=5
[Service]
Type=simple
WorkingDirectory=%h
ExecStart=$exec_path daemon
Environment=$(systemd_quote "HOME=$HOME")
Environment=$(systemd_quote "PATH=$service_path")
Environment=$(systemd_quote "BRIDGET_HOME=$daemon_root")
Environment=$(systemd_quote "BRIDGET_SOCKET=$daemon_socket")
Restart=always
RestartSec=5
TimeoutStopSec=90
[Install]
WantedBy=default.target
EOF
    fi
    if [ -e "$legacy" ] || [ -L "$legacy" ]; then
        legacy_inactive=false
        if [ "$platform" = linux ] && [ ! -L "$legacy" ]; then
            enabled=$(systemctl --user is-enabled bridget-daemon.service 2>/dev/null || :)
            active=$(systemctl --user is-active bridget-daemon.service 2>/dev/null || :)
            if [ "$enabled" = disabled ] && [ "$active" = inactive ]; then legacy_inactive=true; fi
        fi
        $legacy_inactive || fail "An older Bridget service exists but is unavailable: $legacy. Repair that installation before creating another."
    fi
    [ ! -L "$unitdir" ] && [ ! -L "$unit" ] || fail "Service configuration is a symlink: $unit"
    if [ -e "$unit" ]; then
        grep -Fq 'Bridget public installer service v1' "$unit" || fail "An unrecognized service already exists: $unit"
        cmp -s "$work/service" "$unit" || fail "Service settings changed: $unit. Existing configuration was preserved."
    fi
}

service_failure() {
    # Never disable a service that existed before this installation.
    if $service_created; then
        stopped=false
        if [ "$service_kind" = darwin ]; then
            if launchctl bootout "gui/$uid/app.cartae.bridget" >/dev/null 2>&1 && launchctl disable "gui/$uid/app.cartae.bridget" >/dev/null 2>&1; then
                if launchctl print "gui/$uid" >/dev/null 2>&1 && ! launchctl print "gui/$uid/app.cartae.bridget" >/dev/null 2>&1; then stopped=true; fi
            fi
        elif systemctl --user disable --now app.cartae.bridget.service >/dev/null 2>&1; then
            enabled=$(systemctl --user is-enabled app.cartae.bridget.service 2>/dev/null || :)
            active=$(systemctl --user is-active app.cartae.bridget.service 2>/dev/null || :)
            if [ "$enabled" = disabled ] && { [ "$active" = inactive ] || [ "$active" = failed ]; }; then stopped=true; fi
        fi
        if $stopped; then say 'The newly created service was stopped and disabled.'
        else say "Service shutdown could not be confirmed. It may still be active: $unit"; fi
    else
        say "The existing service was not disabled or removed: $unit"
    fi
    fail "$* Installation is not complete; the program and data were retained."
}

start_service() {
    if [ "$service_kind" = existing ]; then
        say 'Bridget is already responding. Its existing service or federation was preserved.'
        say 'Existing automatic startup settings were left unchanged; a manually started daemon was not converted into a service.'
        return
    fi
    mkdir -p "$unitdir" "$daemon_root"
    chmod 700 "$daemon_root"
    if [ ! -e "$unit" ]; then
        staged=$(mktemp "$unitdir/.bridget-service.XXXXXXXX")
        cp "$work/service" "$staged"
        chmod 600 "$staged"
        mv "$staged" "$unit"
        service_created=true
    fi
    if [ "$service_kind" = darwin ]; then
        for log in "$state/service-stdout.log" "$state/service-stderr.log"; do
            [ ! -L "$log" ] || fail "Service log is a symlink: $log"
            touch "$log"
            chmod 600 "$log"
        done
        launchctl enable "gui/$uid/app.cartae.bridget" || service_failure 'macOS refused service activation.'
        if launchctl print "gui/$uid/app.cartae.bridget" >/dev/null 2>&1; then
            launchctl kickstart "gui/$uid/app.cartae.bridget" || service_failure 'macOS could not start Bridget.'
        else
            launchctl bootstrap "gui/$uid" "$unit" || service_failure 'macOS could not load Bridget.'
        fi
    else
        systemctl --user daemon-reload || service_failure 'systemd could not load the service configuration.'
        systemctl --user enable --now app.cartae.bridget.service || service_failure 'systemd could not start Bridget.'
        systemctl --user is-enabled --quiet app.cartae.bridget.service || service_failure 'Automatic startup was not enabled.'
        systemctl --user is-active --quiet app.cartae.bridget.service || service_failure 'Bridget service did not remain active.'
    fi
    attempt=0
    while [ "$attempt" -lt 5 ]; do
        if healthy "$bindir/bridget"; then
            say 'Bridget is running and responding. Automatic startup is enabled.'
            return
        fi
        attempt=$((attempt + 1))
        sleep 1
    done
    service_failure "Bridget did not respond. Inspect the service $unit and its logs; installation is not complete."
}

main() {
    mode=release
    version=
    providers=ask
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --source) mode=source; shift ;;
            --providers)
                [ "$#" -ge 2 ] || fail '--providers needs claude,codex,gemini,cursor or none.'
                [ "$providers" = ask ] || fail '--providers must be supplied only once.'
                providers=$2
                case "$providers" in all) providers=claude,codex,gemini,cursor ;; none) ;; *)
                    case "$providers" in ''|,*|*,|*,,*) fail 'Invalid --providers selection.' ;; esac
                    check_names=$providers
                    check_seen=,
                    check_ifs=$IFS; IFS=,
                    for name in $check_names; do
                        case "$name" in claude|codex|gemini|cursor) ;; *) fail "Unknown provider: $name" ;; esac
                        case "$check_seen" in *,"$name",*) fail "Duplicate provider: $name" ;; esac
                        check_seen="$check_seen$name,"
                    done
                    IFS=$check_ifs ;;
                esac
                shift 2 ;;
            --version)
                [ "$#" -ge 2 ] || fail '--version needs a release tag.'
                version=$2; shift 2 ;;
            --help|-h)
                printf '%s\n' 'Usage: sh install.sh [--source | --version vX.Y.Z] [--providers claude,codex,gemini,cursor|all|none]' \
                    'Default: latest published release. --source: build current main.' \
                    'Interactive terminal: offers missing provider programs. Scripts: explicit --providers, otherwise no provider installation.' \
                    'Installs, starts and verifies a user service; preserves existing running services and accounts.'
                return ;;
            *) fail "Unknown option: $1" ;;
        esac
    done
    [ "$mode" != source ] || [ -z "$version" ] || fail '--source and --version are mutually exclusive.'
    [ "$(id -u)" != 0 ] || fail 'Run as your normal user, not root or sudo.'
    case "${HOME:-}" in /*) ;; *) fail 'HOME must be an absolute directory.' ;; esac
    case "$HOME" in *'
'*) fail 'HOME must not contain a newline.' ;; esac
    HOME=$(cd "$HOME" && pwd -P)
    export HOME
    need curl
    need tar
    command -v sha256sum >/dev/null 2>&1 || need shasum
    os=$(uname -s)
    arch=$(uname -m)
    case "$os" in Darwin) platform=darwin ;; Linux) platform=linux ;; *) fail "Unsupported OS: $os" ;; esac
    case "$arch" in arm64|aarch64) arch=arm64 ;; x86_64|amd64) arch=x86_64 ;; *) fail "Unsupported architecture: $arch" ;; esac
    repo=https://github.com/guthubrx/bridget
    bindir=$HOME/.local/bin
    state=$HOME/.local/state/bridget-install
    envdir=$HOME/.local/share/bridget
    # Reject redirected installation directories. HOME itself may be canonicalized by the OS.
    for dir in "$HOME/.local" "$bindir" "$HOME/.local/state" "$state" "$HOME/.local/share" "$envdir"; do
        [ ! -L "$dir" ] || fail "Installation directory is a symlink: $dir"
    done
    mkdir -p "$bindir" "$state" "$envdir"
    chmod 700 "$state"
    lock=$state/lock
    mkdir "$lock" 2>/dev/null || fail "Another installation is running (or interrupted): inspect $lock"
    work=
    staged=
    new=
    trap 'test -z "$staged" || rm -f "$staged"; test -z "$new" || rm -f "$new"; test -z "$work" || rm -rf "$work"; rm -f "$lock/pid"; rmdir "$lock"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    printf '%s\n' "$$" > "$lock/pid"
    work=$(mktemp -d "$state/stage.XXXXXXXX")
    candidate=$work/bridget
    PATH=$bindir:$PATH
    export PATH

    if [ "$mode" = release ]; then
        if [ -z "$version" ]; then
            latest=$(curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
                --connect-timeout 15 --max-time 60 -o /dev/null -w '%{url_effective}' "$repo/releases/latest")
            case "$latest" in "$repo"/releases/tag/*) version=${latest##*/} ;; *) fail 'Cannot resolve the latest release.' ;; esac
        fi
        case "$version" in *'
'*) fail 'Invalid release tag.' ;; esac
        printf '%s\n' "$version" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.-]+)?$' || fail 'Invalid release tag.'
        stem=bridget-${version#v}-$platform-$arch
        archive=$stem.tar.gz
        say "Downloading $version for $platform-$arch."
        download "$repo/releases/download/$version/$archive.sha256" "$work/checksum" || fail 'No checksum/package for this platform or version. Use --source explicitly if needed.'
        expected=$(awk 'NR == 1 {print $1}' "$work/checksum")
        [ "${#expected}" = 64 ] || fail 'Invalid SHA-256 sidecar.'
        case "$expected" in *[!0-9a-fA-F]*) fail 'Invalid SHA-256 digest.' ;; esac
        expected=$(printf '%s' "$expected" | tr 'A-F' 'a-f')
        download "$repo/releases/download/$version/$archive" "$work/package.tar.gz"
        [ "$(digest "$work/package.tar.gz")" = "$expected" ] || fail 'SHA-256 mismatch. Installed Bridget was not changed.'
        # Stream only the known member, never extract arbitrary archive paths or links.
        tar -xzOf "$work/package.tar.gz" "$stem/bridget" > "$candidate"
        identity=$version
    else
        need git
        need python3
        need cc
        python3 -c 'import sys; sys.exit(sys.version_info < (3, 11))' || fail 'Python 3.11 or later is required for --source.'
        say 'Fetching current main. Source builds take longer than release installs.'
        source=$state/source
        [ ! -L "$source" ] && [ ! -L "$source/.git" ] || fail 'Source checkout must not be a symlink.'
        if [ ! -e "$source" ]; then
            git -c core.hooksPath=/dev/null clone --quiet --depth 1 --single-branch --branch main "$repo.git" "$work/source"
            mv "$work/source" "$source"
        else
            [ -d "$source/.git" ] || fail "Invalid installer checkout: $source"
            [ "$(git -C "$source" remote get-url origin)" = "$repo.git" ] || fail 'Installer checkout has a different origin.'
            source_status=$(git -C "$source" status --porcelain --untracked-files=all) || fail 'Cannot inspect the installer checkout. No update was applied.'
            [ -z "$source_status" ] || fail "Installer checkout was edited: preserve your work in $source before retrying."
            git -C "$source" fetch --quiet --depth 1 origin refs/heads/main
            git -C "$source" -c core.hooksPath=/dev/null checkout --quiet --detach FETCH_HEAD
        fi
        identity=$(git -C "$source" rev-parse HEAD)
        export PATH="$HOME/.cargo/bin:$PATH"
        BRIDGET_BUILD_ID=$identity
        export BRIDGET_BUILD_ID
        unset RUSTUP_TOOLCHAIN
        if ! command -v rustup >/dev/null 2>&1; then
            download https://sh.rustup.rs "$work/rustup.sh"
            sh "$work/rustup.sh" -y --profile minimal --no-modify-path --default-toolchain none
        fi
        need cargo
        # Same registered cache on every invocation; the existing wrapper owns its budget.
        CARGO_TARGET_DIR=$HOME/.cache/bridget-build-installer
        export CARGO_TARGET_DIR
        (cd "$source" && python3 scripts/build.py cargo build --locked --release -p bridget-daemon)
        cp "$CARGO_TARGET_DIR/release/bridget" "$candidate"
    fi
    [ -s "$candidate" ] || fail 'Package contains no Bridget executable.'
    chmod 755 "$candidate"
    "$candidate" --version > "$work/version" || fail 'Candidate cannot run. Installed Bridget was not changed.'
    grep -qi 'bridget' "$work/version" || fail 'Unexpected candidate version response.'
    prepare_providers
    # Ask the candidate before changing a link, profile or service. Existing clients were preserved.
    if "$candidate" clients check 2>"$work/client-check-error"; then
        :
    else
        client_check_status=$?
        # Published 0.6.3 predates this port. Preserve its historical installation path,
        # with an explicit warning, without calling an unknown version compatible.
        if [ "$mode" = release ] && [ "$version" = v0.6.3 ] && [ "$client_check_status" = 2 ] && grep -Eq '^bridget 0[.]6[.]3([[:space:]]|$)' "$work/version"; then
            say 'Legacy release 0.6.3: provider-client compatibility is NOT verified by this package. This preserves its historical installation behavior. Use --source for the current client-version check. No provider client will be updated, and model access is not guaranteed.'
        else
            fail 'Client compatibility check refused this Bridget installation. Keep existing services running. Read the machine, installed/minimum versions and action above. Ask the user before updating Claude Code or Codex through its existing official installer; then repeat this installation. No account or model change is authorized by this refusal.'
        fi
    fi
    prepare_service

    # Prepare PATH before replacing the executable; preserve existing shell settings.
    [ ! -L "$envdir/env" ] || fail "Symlink refused: $envdir/env"
    {
        printf '%s\n' 'case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) export PATH="$HOME/.local/bin:$PATH" ;; esac'
        printf 'export BRIDGET_HOME=%s\n' "$(shell_literal "$daemon_root")"
        printf 'export BRIDGET_SOCKET=%s\n' "$(shell_literal "$daemon_socket")"
        if [ -f "$daemon_root/federation.env" ]; then printf '%s\n' 'export BRIDGET_CHANNEL=ssh-unix'; fi
    } > "$envdir/env"
    profiles="$HOME/.profile"
    case "${SHELL:-}" in
        */zsh) profiles="$profiles
$HOME/.zshrc
$HOME/.zprofile" ;;
        */bash) profiles="$profiles
$HOME/.bashrc"
            # Do not create .bash_profile: it would hide existing .profile/.bash_login.
            if [ -e "$HOME/.bash_profile" ] || [ -L "$HOME/.bash_profile" ]; then
                profiles="$profiles
$HOME/.bash_profile"
            elif [ -e "$HOME/.bash_login" ] || [ -L "$HOME/.bash_login" ]; then
                profiles="$profiles
$HOME/.bash_login"
            fi ;;
    esac
    oldifs=$IFS
    IFS='
'
    # Validate every destination before changing the first profile.
    for profile in $profiles; do
        [ ! -L "$profile" ] || fail "Shell profile is a symlink: $profile. Configure PATH manually."
    done
    for profile in $profiles; do
        if ! grep -Fq '. "$HOME/.local/share/bridget/env"' "$profile" 2>/dev/null; then
            if [ -f "$profile" ] && [ ! -e "$profile.before-bridget-install" ] && [ ! -L "$profile.before-bridget-install" ]; then
                cp -p "$profile" "$profile.before-bridget-install"
            fi
            staged=$(mktemp "$profile.bridget.XXXXXXXX")
            if [ -f "$profile" ]; then cp -p "$profile" "$staged"; fi
            { printf '%s\n' '. "$HOME/.local/share/bridget/env"'; test ! -f "$profile" || cat "$profile"; } > "$staged"
            mv -f "$staged" "$profile"
        fi
    done
    IFS=$oldifs
    [ ! -d "$bindir/bridget" ] || fail "Destination is a directory: $bindir/bridget"
    if [ -e "$bindir/bridget" ] && [ "$(digest "$bindir/bridget")" != "$(digest "$candidate")" ]; then
        cp -pL "$bindir/bridget" "$work/previous"
        mv -f "$work/previous" "$state/previous-bridget"
    fi
    # Stage on the destination filesystem, then rename without following the old symlink.
    new=$(mktemp "$bindir/.bridget-install.XXXXXXXX")
    if ! cp "$candidate" "$new" || ! chmod 755 "$new" || ! mv -f "$new" "$bindir/bridget"; then
        rm -f "$new"
        fail 'Could not replace the executable.'
    fi
    printf '%s\n' "channel=$mode" "source=$identity" "sha256=$(digest "$bindir/bridget")" > "$work/installed.txt"
    mv -f "$work/installed.txt" "$state/installed.txt"
    start_service
    if [ "$mode" = source ]; then
        (cd "$source" && python3 scripts/build.py installed --target "$CARGO_TARGET_DIR" --binary "$bindir/bridget") || fail "The executable was installed at $bindir/bridget, but build-cache cleanup failed. Inspect $CARGO_TARGET_DIR; services were not restarted."
    fi
    say "Installed $(cat "$work/version") ($identity) at $bindir/bridget"
    say 'For this terminal: . "$HOME/.local/share/bridget/env"'
    say 'Start an agent in your project with: bridget claude or bridget codex'
    say 'Existing running services were not interrupted. Provider login remains personal.'
    say 'Claude/Codex accounts and Bridget data were preserved.'
}

# Run a complete function: a truncated download cannot execute half an installation.
main "$@"
