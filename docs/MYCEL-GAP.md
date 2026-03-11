# The Gap Mycel Fills

Status: explanatory note

This document explains why protocols like Mycel have been relatively rare, and how the problems Mycel is trying to address are usually handled today.

## 1. Why a Protocol Like Mycel Has Been Rare

Mycel is trying to solve an awkward in-between problem:

- stricter than ordinary document collaboration
- less interested in global single-consensus than blockchains
- more concerned than Git with the reader-side default accepted result
- more concerned than a typical app backend with verifiable history and portability

Needs like this have existed for a long time, but they usually have not concentrated into one standalone protocol market, so they have mostly been handled in fragmented ways.

### 1.1 The problem is distributed, not a concentrated pain point in one industry

What most users need first is:

- a usable editor
- document versioning
- a centralized backend
- web collaboration

Very few users start by asking for all of the following at once:

- verifiable history
- multiple valid branches
- a default reading derived from fixed rules
- decentralized replication

### 1.2 Existing solutions each absorb only part of the need

Because of that, the market does not naturally force a middle-layer protocol like Mycel into existence.

### 1.3 This kind of protocol is harder to explain in one sentence

Git is easy to explain, and blockchains are easy to describe.

But something like Mycel, which is "not Git, not a chain, but a protocol for verifiable text history and governed reading", is harder to understand at a glance.

### 1.4 The engineering cost is high, while short-term commercial return is unclear

Building a system like Mycel means handling all of these at once:

- canonicalization
- signed objects
- replay
- accepted-state rules
- partial replication
- wire sync

That is a substantial engineering burden, but unlike chat apps, payment systems, or content platforms, it is not obviously easy to monetize quickly.

## 2. How These Problems Are Usually Handled Today

### 2.1 Centralized platforms

The most common approach is to use systems like:

- Google Docs
- Notion
- Confluence
- CMS and knowledge-base platforms

These systems usually:

- keep all state in the platform database
- let the platform decide the default version
- expose history, but usually without an independently verifiable and portable history model

Advantages:

- easy to use
- quick to deploy
- low learning cost

Limits:

- strong platform dependence
- history is hard to verify independently
- branch and governance models are usually weak

### 2.2 Git

Another common approach is to treat text as a Git repository.

That gives:

- branches
- commit history
- merge behavior
- review flow

Advantages:

- very good for authors and maintainers
- strong history preservation
- mature multi-branch model

Limits:

- more of an author tool than a reader tool
- "how the default reading is derived" is not one of its core design goals
- app, governance, and accepted-state semantics do not fit naturally

### 2.3 Blockchains or smart contracts

Some systems put important state on-chain and use consensus to determine one unique state.

Advantages:

- strong consistency
- high verifiability
- one shared global state

Limits:

- heavy
- expensive
- overdesigned for many text-governance cases
- many situations simply do not need one unique network-wide truth

### 2.4 Custom backends and databases

In practice, the most common approach is still to build an application backend:

- state in a database
- audit through logs
- permissions, reading results, and history implemented inside the application backend

Advantages:

- most practical
- easiest to integrate with an existing product
- fastest to ship

Limits:

- rarely interoperable
- poor portability
- almost no protocol-level verification

## 3. What Gap Mycel Is Trying To Fill

Mycel is trying to fill the layer between centralized platforms and global-consensus chains:

- history should be verifiable
- default accepted results should be explainable
- multiple valid branches should be able to coexist
- without paying the cost of blockchain-style global consensus

In other words, Mycel is trying to provide:

- verifiable history
- a default reading derived from fixed rules
- decentralized replication
- without requiring the whole network to collapse into one unique truth

It is not trying to replace every existing solution.

It is trying to turn a scattered set of needs into a layered, specifiable, and interoperable protocol design.

## 4. One-sentence Version

The need has existed for a long time, but it was absorbed piecemeal by centralized platforms, Git, blockchains, and custom backends. Mycel matters because it recomposes those scattered needs into one protocol for verifiable history, governed default reading, and decentralized replication.
