# TODO CHECKLIST for haystack-rs rust port

PREAMBLE PROMPT: HIGH THINKING:
Your prime directive is to convert all of the haystack dir (python code) to haystack-rs dir (rust code) for complete feature parity (to the point that it makes sense for the user) at the edge api level. The python and rust code will never interact with each other, so don't worry about making them interoperable across language.

## end-2-end tests for seeing how the public api is used

- python code for reference is at the repo root: e2e/pipelines
- rust directory for porting is at the rust project root: ./haystack-rs/haystack-rs/e2e-rs

## High Level module parity completion trackers

- [] components
- [] core
- [] data
- [] dataclasses
- [] document_stores
- [] evaluation
- [] marshal
- [] testing
- [] telemetry
- [] tools
- [] tracing
- [] utils

## Individual Modules

Does not track down to the individual file level, just at the mod (folder) level so that you are not too restricted.

### haystack-rs/components module

- [] agents
- [] audio
- [] builders
- [] caching
- [] classifiers
- [] connectors
- [] converters
- [] embedders
  - [] backends
- [] evaluators
- [] extractors
- [] fetchers
- [] generators
- [] joiners
- [] mod.rs
- [] preprocessors
- [] rankers
- [] readers
- [] retrievers
  - [] in_memory
- [] routers
- [] samplers
- [] tools
- [] validators
- [] websearch
- [] writers

### core

component
pipeline
super_component

### data

abbreviations

### dataclasses

answer.rs
chat_message.rs
lib.rs
state.rs
byte_stream.rs
document.rs
sparse_embedding.rs
streaming_chunk.rs

### document_stores

errors
in_memory
types

### evaluation

### marshal

### testing

callable_serialization
sample_components

### telemetry

### tools

### tracing

### utils
