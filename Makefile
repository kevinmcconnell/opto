PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
DATADIR := $(PREFIX)/share
UNITDIR ?= $(HOME)/.config/systemd/user
APP_ID := io.github.kevinmcconnell.Opto

.PHONY: build test install install-files uninstall

build:
	cargo build --release --locked

test:
	cargo test --workspace

install: install-files
	$(BINDIR)/opto setup service
	-$(BINDIR)/opto setup claude

install-files: build
	install -Dm755 target/release/opto-collect $(DESTDIR)$(BINDIR)/opto-collect
	install -Dm755 target/release/opto $(DESTDIR)$(BINDIR)/opto
	install -Dm755 data/opto-claude-statusline $(DESTDIR)$(BINDIR)/opto-claude-statusline
	install -dm755 $(DESTDIR)$(UNITDIR)
	sed 's|@bindir@|$(BINDIR)|' data/opto-collect.service.in >$(DESTDIR)$(UNITDIR)/opto-collect.service
	install -Dm644 data/$(APP_ID).desktop $(DESTDIR)$(DATADIR)/applications/$(APP_ID).desktop
	install -Dm644 data/$(APP_ID).svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	install -Dm644 LICENSE $(DESTDIR)$(DATADIR)/licenses/opto/LICENSE

uninstall:
	-$(BINDIR)/opto remove
	rm -f $(BINDIR)/opto-collect $(BINDIR)/opto $(BINDIR)/opto-claude-statusline
	rm -f $(UNITDIR)/opto-collect.service
	rm -f $(DATADIR)/applications/$(APP_ID).desktop
	rm -f $(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	rm -rf $(DATADIR)/licenses/opto
	systemctl --user daemon-reload
