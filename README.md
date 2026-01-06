# Newsletter Service

**A production-oriented Rust backend service for managing newsletter subscriptions and email delivery, built with async execution, explicit failure handling, and observability in mind.**

---

## Table of Contents
1. [Overview](#overview)  
2. [Features](#features)  
3. [Architecture & Design Decisions](#architecture--design-decisions)  
4. [Getting Started](#getting-started)  
5. [Operational Considerations](#operational-considerations)  
6. [Acknowledgements](#acknowledgements)  

---

## Overview
This project implements a **production-grade newsletter backend** in Rust. It is designed to handle:

- Subscription management
- Confirmation workflows
- Async email delivery
- Observability and logging
- Fault-tolerant, predictable behavior under load

**This project prioritizes correctness, explicit failure handling, and predictable performance over rapid iteration.**

It is intended to demonstrate how to build robust, production-ready systems in Rust, suitable for real-world workloads.

---

## Features
- Async HTTP APIs for subscription management  
- Persistent storage via PostgreSQL  
- Background tasks for email delivery  
- Structured logging and observability hooks  
- Integration and unit tests for critical workflows  
- Idempotent operations to prevent duplicate emails  

---

## Architecture & Design Decisions
- **Async-first design:** All IO operations use `tokio` async runtime for scalability.  
- **Application state container:** Shared resources (DB pool, configuration) wrapped in `Arc` for safe concurrent access.  
- **Error handling strategy:** Explicit `Result` types; errors are logged and surfaced to observability system.  
- **Background email delivery:** Tasks are scheduled asynchronously, with retry logic for failed sends.  
- **Idempotency:** Subscription endpoints ensure duplicate requests do not result in multiple emails.  
- **Test strategy:** Unit tests for core logic; integration tests for DB and email delivery flows.  
- **Extensions beyond reference architecture:** Added idempotency, retry logic, and observability hooks to simulate real production requirements.

---

## Getting Started

### Prerequisites
- Rust 1.72+  
- PostgreSQL  
- Cargo & `rustfmt` / `clippy` for linting  
- `.env` file for configuration (DB URL, email credentials)

### Installation
```bash
git clone https://github.com/yourusername/newsletter-service.git
cd newsletter-service
cargo build

## Acknowledgements
Initial architecture and learning reference inspired by *Zero to Production in Rust* by Luca Palmieri.  
All design decisions, implementations, and enhancements beyond the reference were made independently to reinforce understanding and simulate production constraints.
