#!/bin/bash

mdbook clean
mdbook build
rsync -rlv book/ fatcat-qa-vm:/srv/fatcat/guide
