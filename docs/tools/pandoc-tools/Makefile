.PHONY: all build clean set-permissions set-timestamps

all: clean build

build: css/default.css set-permissions set-timestamps

css-dir:
	mkdir -p css

css/default.css: | css-dir
	curl \
		-o $@ \
		https://raw.githubusercontent.com/sindresorhus/github-markdown-css/gh-pages/github-markdown.css

set-permissions:
	find . -type d -exec chmod 755 {} +
	find . -type f -exec chmod 644 {} +

set-timestamps:
	find . -path './.git' -prune -o -exec touch {} +

clean:
	rm -fr css
