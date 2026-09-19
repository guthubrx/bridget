.PHONY: build release test daemon install uninstall clean clean-builds test-build-cleanup

BINARY = bridget
INSTALL_DIR = $(HOME)/.local/bin
LAUNCHD_PLIST = $(HOME)/Library/LaunchAgents/com.bridget.daemon.plist
export CARGO_TARGET_DIR ?= $(CURDIR)/target
RELEASE_BIN = $(CARGO_TARGET_DIR)/release/$(BINARY)

build:
	python3 scripts/build.py cargo build

release:
	python3 scripts/build.py cargo build --release

test:
	python3 scripts/build.py cargo test

daemon: release
	RUST_LOG=info "$(RELEASE_BIN)" daemon

install: release
	@echo "Installation du binaire bridget..."
	install -d $(INSTALL_DIR)
	install -m 755 "$(RELEASE_BIN)" "$(INSTALL_DIR)/$(BINARY)"
	@echo "Installation du service launchd..."
	install -d $(dir $(LAUNCHD_PLIST))
	@python3 -c "import os; home=os.path.expanduser('~'); print(f'''\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
    <key>Label</key><string>com.bridget.daemon</string>\n\
    <key>ProgramArguments</key>\n\
    <array>\n\
        <string>{home}/.local/bin/bridget</string>\n\
        <string>daemon</string>\n\
    </array>\n\
    <key>EnvironmentVariables</key>\n\
    <dict>\n\
        <key>RUST_LOG</key><string>info</string>\n\
        <key>HOME</key><string>{home}</string>\n\
    </dict>\n\
    <key>RunAtLoad</key><true/>\n\
    <key>KeepAlive</key><true/>\n\
    <key>StandardOutPath</key><string>{home}/.cache/bridget/daemon-stdout.log</string>\n\
    <key>StandardErrorPath</key><string>{home}/.cache/bridget/daemon-stderr.log</string>\n\
</dict>\n\
</plist>''')" > "$(LAUNCHD_PLIST)"
	launchctl load $(LAUNCHD_PLIST) 2>/dev/null || true
	@echo ""
	@echo "Bridget installé !"
	@echo "  Binaire : $(INSTALL_DIR)/$(BINARY)"
	@echo "  Service : launchd (com.bridget.daemon)"
	@echo ""
	@echo "Usage :"
	@echo "  bridget codex        Lance Codex + connexion daemon"
	@echo "  bridget claude       Lance Claude + connexion daemon"
	@echo "  bridget send --to N  Envoie un message"
	@echo "  bridget who          Liste les agents"
	@echo "  bridget daemon       Lance le daemon manuellement"
	python3 scripts/build.py installed --target "$(CARGO_TARGET_DIR)" --binary "$(INSTALL_DIR)/$(BINARY)"

uninstall:
	@echo "Arrêt du daemon..."
	launchctl unload $(LAUNCHD_PLIST) 2>/dev/null || true
	@echo "Suppression du binaire..."
	rm -f $(INSTALL_DIR)/$(BINARY)
	rm -f $(LAUNCHD_PLIST)
	@echo "Bridget désinstallé."

clean:
	python3 scripts/build.py clean

clean-builds:
	python3 scripts/build.py clean $(if $(DRY_RUN),--dry-run,)

test-build-cleanup:
	python3 -m unittest discover -s scripts/tests -p 'test_build_cleanup.py' -v
