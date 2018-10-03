#!/bin/bash

mdbook clean
mdbook build
rsync -arv book/ fatcat-prod1-vm:/srv/fatcat/guide
