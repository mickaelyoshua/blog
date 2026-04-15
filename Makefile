BROWSER ?= $(shell command -v chromium || command -v chromium-browser || command -v google-chrome-stable || command -v google-chrome || command -v brave-browser)
PORT ?= 3000
URL := http://127.0.0.1:$(PORT)/cv
OUT ?= resume.pdf

.PHONY: resume pdf clean-pdf
.PRECIOUS: $(OUT)

resume: $(OUT)

pdf: $(OUT)

$(OUT):
	@if [ -z "$(BROWSER)" ]; then \
		echo "error: no Chromium-based browser found (set BROWSER=/path/to/chrome)"; exit 1; \
	fi
	@echo "using browser: $(BROWSER)"
	@cargo build --quiet
	@./target/debug/blog & \
	SERVER_PID=$$!; \
	trap "kill $$SERVER_PID 2>/dev/null" EXIT INT TERM; \
	for i in $$(seq 1 40); do \
		curl -sSf $(URL) >/dev/null 2>&1 && break; sleep 0.25; \
	done; \
	$(BROWSER) --headless=new --disable-gpu \
		--no-pdf-header-footer --print-to-pdf-no-header \
		--no-sandbox --disable-sync --disable-extensions \
		--disable-background-networking --disable-default-apps \
		--no-pings --no-first-run --no-default-browser-check \
		--virtual-time-budget=5000 \
		--print-to-pdf=$(OUT) $(URL); \
	kill $$SERVER_PID 2>/dev/null; wait $$SERVER_PID 2>/dev/null; true
	@echo "wrote $(OUT)"

clean-pdf:
	rm -f $(OUT)
