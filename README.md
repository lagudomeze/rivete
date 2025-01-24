# Rivete

## Introduction

Rivete, inspired by the word "Rivet," symbolizes the solid connections made possible through Dependency Injection (DI). The name aligns with the core concept of Inversion of Control (IoC) and features a natural extension with `.rs` for Rust projects. Although "Rivet" was already taken, "Rivete" preserves the essence while offering a unique identity.

Rivete aims to provide a lightweight, compile-time IoC framework for Rust, drawing inspiration from the simplicity and extensibility of Spring Boot. The framework is currently in its early stages (version 0.0.1).

## Project Goals

1. **Zero-Cost Abstraction**: Minimize runtime overhead and leverage Rust’s compile-time guarantees wherever possible.
2. **Compile-Time Execution**: Use Rust’s metaprogramming capabilities to analyze and generate code at compile time.
3. **Extensibility**: Offer a modular architecture to support features like MVC, scheduling, database integrations, declarative HTTP clients, and background task handling.

## Core Features

### Static Initialization

- Implements a system akin to `OnceLock` but ensures as much zero-cost access to initialized static values as possible.
- Guarantees safety for accessing initialized values with compile-time verification using an `Inited` marker.
- During IoC container initialization, a `Map<TypeId, Inited>` ensures each bean is initialized exactly once.
- After initialization, lightweight markers (cloneable, copyable, thread-safe, and zero-sized) indicate ready-to-use components.

### Compile-Time Bean Discovery

- Leverages the `syn` crate to scan all code within the crate during compilation.
- Automatically detects and registers beans based on Rust’s module system.
- Constructs initialization functions or types for discovered beans.

### Extensibility

- Modular system supports additional features, including:
  - **MVC**: Automatically collect and register HTTP interfaces.
  - **Scheduling**: Simplify task scheduling with declarative annotations.
  - **Database Integration**: Provide wrappers similar to MyBatis for seamless data access.
  - **Declarative HTTP Clients**: Inspired by Feign, define HTTP clients declaratively.
  - **Background Processing**: Handle Kafka consumers and similar asynchronous tasks with ease.

## Getting Started

This project is under active development. The current version (0.0.1) focuses on laying the groundwork for the IoC container and static initialization system.

To contribute or explore the codebase:

1. Clone the repository.
2. Experiment with defining beans and initializing the IoC container.
3. Share feedback or request new features by opening an issue.

## Future Plans

- Enhance documentation with detailed usage examples.
- Implement advanced features like scheduling, MVC support, and declarative HTTP clients.
- Build a vibrant community around Rivete for collaborative development.

---

For questions, contributions, or feature requests, please open an issue on the project repository. Together, let’s make Rivete a powerful, as much zero-cost IoC framework for Rust!
