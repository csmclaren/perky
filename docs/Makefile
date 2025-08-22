.PHONY: all build clean

NAME := perky

REF := main

FILTER_DIR := tools/pandoc-tools/filters
TEMPLATE_DIR := tools/pandoc-tools/templates

all: clean build

build: \
	build/$(NAME).md \
	build/$(NAME).css \
	build/$(NAME).html \
	build/$(NAME)-standalone.html \
	build/README.md

build-dir:
	mkdir -p build

build/$(NAME).md: | build-dir
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/append-default-footer.lua \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--output $@ \
		--to gfm \
		--wrap none \
		src/$(NAME).md

build/$(NAME).css: | build-dir
	cp tools/pandoc-tools/css/default.css $@

build/$(NAME).html: | build-dir
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
		src/$(NAME).md

build/$(NAME)-standalone.html: | build-dir
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
		src/$(NAME).md

build/README.md: | build-dir
	pandoc \
		--from gfm \
		--lua-filter=$(FILTER_DIR)/toc.lua \
		--output $@ \
		--to gfm \
		--wrap none \
		src/README.md

clean:
	rm -fr build
