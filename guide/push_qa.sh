#!/bin/bash

mdbook clean
mdbook build
rsync -arv book/ fatcat-qa-vm:/srv/fatcat/guide
