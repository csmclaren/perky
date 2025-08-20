.PHONY: all build clean docs set-permissions set-timestamps clean-resources collect-resources

all: clean build

build: docs README.md set-permissions set-timestamps

docs:
	$(MAKE) -C docs build

README.md: docs
	cp docs/build/$@ $@

set-permissions:
	find . -type d -exec chmod 755 {} +
	find . -type f -exec chmod 644 {} +

set-timestamps:
	find . -path './.git' -prune -o -exec touch {} +

clean:
	$(MAKE) -C docs clean
	rm -f README.md

clean-resources:
	rm -fr resources

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
	resources/charfreq-shakespeare/3-grams.tsv \
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
