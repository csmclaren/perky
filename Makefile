.PHONY: all build check-jq check-python clean docs docs-build-dir set-permissions set-timestamps clean-resources collect-resources

NAME := perky
REF := main

FILTER_DIR := tools/pandoc-tools/filters
TEMPLATE_DIR := tools/pandoc-tools/templates

all: clean build

build: docs README.md set-permissions set-timestamps

check-jq:
	@command -v jq >/dev/null 2>&1 || \
	  { echo "Error: 'jq' is not installed or not in PATH."; exit 1; }

check-python:
	@command -v python3 >/dev/null 2>&1 || \
	  { echo "Error: 'python3' is not installed or not in PATH."; exit 1; }
	@python3 -c 'import sys; exit(0 if sys.version_info >= (3,11) else 1)' || \
	  { echo "Error: Python 3.11 or higher is required."; exit 1; }

clean:
	$(RM) -f README.md
	$(RM) -fr docs/build

docs: \
	docs/build/$(NAME)-standalone.html \
	docs/build/$(NAME).css \
	docs/build/$(NAME).html \
	docs/build/$(NAME).md \
	docs/build/README.md

docs-build-dir:
	mkdir -p docs/build

docs/build/$(NAME)-standalone.html: docs/src/$(NAME).md docs/build/$(NAME).css | docs-build-dir
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/append-html-footer.lua \
		--lua-filter=$(FILTER_DIR)/embed-stylesheet.lua \
		--lua-filter=$(FILTER_DIR)/process-github-links.lua \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--metadata filter_process_github_links.ref=$(REF) \
		--output $@ \
		--template $(TEMPLATE_DIR)/default.html \
		--to html \
		--wrap none \
		docs/src/$(NAME).md

docs/build/$(NAME).css: tools/pandoc-tools/css/default.css | docs-build-dir
	cp tools/pandoc-tools/css/default.css $@

docs/build/$(NAME).html: docs/src/$(NAME).md | docs-build-dir docs/build/$(NAME).css
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/append-html-footer.lua \
		--lua-filter=$(FILTER_DIR)/link-stylesheet.lua \
		--lua-filter=$(FILTER_DIR)/process-github-links.lua \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--metadata filter_process_github_links.ref=$(REF) \
		--output $@ \
		--template $(TEMPLATE_DIR)/default.html \
		--to html \
		--wrap none \
		docs/src/$(NAME).md

docs/build/$(NAME).md: docs/src/$(NAME).md | docs-build-dir
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/append-default-footer.lua \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--output $@ \
		--to gfm \
		--wrap none \
		docs/src/$(NAME).md

docs/build/README.md: docs/src/README.md | docs-build-dir
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--output $@ \
		--to gfm \
		--wrap none \
		docs/src/README.md

README.md: docs/build/README.md
	cp docs/build/$@ $@

set-permissions:
	find . -type d -exec chmod 755 {} +
	find . -type f -exec chmod 644 {} +

set-timestamps:
	find . -path './.git' -prune -o -exec touch {} +

clean-resources:
	$(RM) -fr resources

collect-resources: \
	resources/charfreq-dfko/1-grams-uc.tsv \
	resources/charfreq-dfko/1-grams.tsv \
	resources/charfreq-google/1-grams-uc.tsv \
	resources/charfreq-google/2-grams-uc.tsv \
	resources/charfreq-google/3-grams-uc.tsv \
	resources/charfreq-linux/1-grams-uc.tsv \
	resources/charfreq-linux/1-grams.tsv \
	resources/charfreq-linux/2-grams-uc.tsv \
	resources/charfreq-linux/2-grams.tsv \
	resources/charfreq-linux/3-grams-uc.tsv \
	resources/charfreq-shakespeare/1-grams-uc.tsv \
	resources/charfreq-shakespeare/1-grams.tsv \
	resources/charfreq-shakespeare/2-grams-uc.tsv \
	resources/charfreq-shakespeare/2-grams.tsv \
	resources/charfreq-shakespeare/3-grams-uc.tsv \
	resources/charfreq-shakespeare/3-grams.tsv

resources/charfreq-dfko:
	mkdir -p $@

resources/charfreq-dfko/%-grams.tsv: ../charfreq-dfko/output/%-grams.tsv | resources/charfreq-dfko
	cp $< $@

resources/charfreq-dfko/%-grams-uc.tsv: ../charfreq-dfko/output/%-grams-uc.tsv | resources/charfreq-dfko
	cp $< $@

resources/charfreq-google:
	mkdir -p $@

resources/charfreq-google/%-grams-uc.tsv: ../charfreq-google/output/%-grams-uc.tsv | resources/charfreq-google
	cp $< $@

resources/charfreq-linux:
	mkdir -p $@

resources/charfreq-linux/%-grams.tsv: ../charfreq-linux/output/%-grams.tsv | resources/charfreq-linux
	cp $< $@

resources/charfreq-linux/%-grams-uc.tsv: ../charfreq-linux/output/%-grams-uc.tsv | resources/charfreq-linux
	cp $< $@

resources/charfreq-shakespeare:
	mkdir -p $@

resources/charfreq-shakespeare/%-grams.tsv: ../charfreq-shakespeare/output/%-grams.tsv | resources/charfreq-shakespeare
	cp $< $@

resources/charfreq-shakespeare/%-grams-uc.tsv: ../charfreq-shakespeare/output/%-grams-uc.tsv | resources/charfreq-shakespeare
	cp $< $@
