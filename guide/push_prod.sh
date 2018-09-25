#!/bin/bash

mdbook clean
mdbook build
rsync -arv book/ fatcat-prod-vm:/srv/fatcat/guide
