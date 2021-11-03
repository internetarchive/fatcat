#!/usr/bin/env python3

import argparse
import sys

import raven

from fatcat_tools import authenticated_api
from fatcat_tools.reviewers import DummyReviewBot

# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = raven.Client()


def run_dummy(args):
    reviewer = DummyReviewBot(args.api, poll_interval=args.poll_interval,
        verbose=args.verbose)
    if args.editgroup:
        annotation = reviewer.run_single(args.editgroup, args.annotate)
        print(annotation)
    elif args.continuous:
        reviewer.run()

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--verbose',
        action='store_true',
        help="enable verbose output")
    parser.add_argument('--fatcat-api-url',
        default="http://localhost:9411/v0",
        help="fatcat API host/port to use")
    parser.add_argument('--poll-interval',
        help="how long to wait between polling (seconds)",
        default=10.0, type=float)
    subparsers = parser.add_subparsers()

    sub_dummy = subparsers.add_parser('dummy',
        help="example/demonstration review bot")
    sub_dummy.set_defaults(func=run_dummy)
    sub_dummy.add_argument("--continuous",
        action="store_true",
        help="run forever, polling for new reviewable editgroups")
    sub_dummy.add_argument("--editgroup",
        help="single editgroup ID to review")
    sub_dummy.add_argument("--annotate",
        action="store_true",
        help="for single editgroups, pushes result as annotation")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)
    if (args.editgroup and args.continuous) or not (args.editgroup or args.continuous):
        print("need to run on a single editgroup, or continuous")
        sys.exit(-1)

    args.api = authenticated_api(args.fatcat_api_url)
    args.func(args)

if __name__ == '__main__':
    main()
