PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
DATADIR := $(PREFIX)/share
UNITDIR := $(HOME)/.config/systemd/user
APP_ID := io.github.kevinmcconnell.Opto

.PHONY: build test install uninstall

build:
	cargo build --release

test:
	cargo test --workspace

install: build
	install -Dm755 target/release/opto-collect $(BINDIR)/opto-collect
	install -Dm755 target/release/opto $(BINDIR)/opto
	install -Dm755 data/opto-claude-statusline $(BINDIR)/opto-claude-statusline
	install -Dm644 data/opto-collect.service $(UNITDIR)/opto-collect.service
	install -Dm644 data/$(APP_ID).desktop $(DATADIR)/applications/$(APP_ID).desktop
	install -Dm644 data/$(APP_ID).svg $(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	$(BINDIR)/opto setup service
	-$(BINDIR)/opto setup claude

uninstall:
	-$(BINDIR)/opto remove
	rm -f $(BINDIR)/opto-collect $(BINDIR)/opto $(BINDIR)/opto-claude-statusline
	rm -f $(UNITDIR)/opto-collect.service
	rm -f $(DATADIR)/applications/$(APP_ID).desktop
	rm -f $(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	systemctl --user daemon-reload
