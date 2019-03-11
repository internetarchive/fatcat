#!/bin/bash

mdbook clean
mdbook build
rsync -rlv book/ fatcat-prod1-vm:/srv/fatcat/guide
