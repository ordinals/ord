# Minimal Sphinx Makefile for GitHub Actions

SPHINXBUILD   = sphinx-build
SOURCEDIR     = .
BUILDDIR      = build

.PHONY: help clean html

help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS)

clean:
	rm -rf $(BUILDDIR)/*

html:
	@$(SPHINXBUILD) -M html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS)
