# TODO CHECKLIST for haystack-rs rust port

PREAMBLE PROMPT: HIGH THINKING:
Your prime directive is to convert all of the haystack dir (python code) to haystack-rust dir (rust code) for complete feature parity (to the point that it makes sense for the user) at the edge api level. The python and rust code will never interact with each other, so don't worry about making them interoperable across language.

## end-2-end tests for seeing how the public api is used

- rust directory for porting python to is at the rust project root: haystack-rs/haystack-rust/e2e-rs
- python code for reference is at the repo root: haystack-rs/e2e/pipelines

## High Level module parity completion trackers

These completion checklists throughout the document perfectly mirror the original directory layout python project layout (in ../haystack-rs/haystack) so you don't get confused. When working in a module, scroll down to the checklist for that module and fill it out. Only mark something complete here (at the high level) if the entire submodule is complete.

Checklist:

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

Does not track down to the individual file level, just at the mod (folder) level. You should fill these in as you complete tasks

### haystack-rs/components module

Description: Component system, pipeline, serialization, errors

Checklist:

- [x] (module root)
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
- [x] joiners
- [x] mod.rs
- [x] preprocessors
- [] rankers
- [] readers
- [] retrievers
  - [x] in_memory
- [] routers
- [] samplers
- [] tools
- [] validators
- [] websearch
- [] writers

### core

Checklist:

- [x] (module root)
- [x] component
- [] pipeline
- [] super_component

### data

- [] (module root)
- [] abbreviations

### dataclasses

- [x] (module root)
- [x] answer.rs
- [x] chat_message.rs
- [x] lib.rs
- [x] state.rs
- [x] byte_stream.rs
- [x] document.rs
- [x] sparse_embedding.rs
- [x] streaming_chunk.rs

### document_stores

- [x] (module root)
- [] errors
- [x] in_memory
- [x] types

### evaluation

- [] (module root)

### marshal

- [] (module root)

### testing

- [] (module root)
- [] callable_serialization
- [] sample_components

### telemetry

- [] (module root)

### tools

- [] (module root)

### tracing

- [] (module root)

### utils

- [] (module root)
