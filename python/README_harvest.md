
## State Refactoring

Harvesters should/will work on fixed window sizes.

Serialize state as JSON, publish to a state topic. On load, iterate through the
full state topic to construct recent history, and prepare a set of windows that
need harvesting, then iterate over these.

If running as continuous process, will retain state and don't need to
re-iterate; if cron/one-off, do need to re-iterate.

To start, do even OAI-PMH as dates.

## "Bootstrapping" with bulk metadata

1. start continuous update harvesting at time A
2. do a bulk dump starting at time B1 (later than A, with a margin), completing at B2
3. with database starting from scratch at C (after B2), load full bulk
   snapshot, then run all updates since A

