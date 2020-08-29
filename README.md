<div align="center">

  <h1><code>snakeriver-server</code></h1>

  <strong>Graph Server for the SnakeRiver System</strong>

  <h3>
    <a href="https://github.com/JusungLee0601/snakeriver-clientside">snakeriver-clientside</a>
  <br>
  <a href="https://docs.google.com/presentation/d/1ODsMf6o6zTsH2Zp_oWeKQSeTu0nmWtI7VzPwBKDrOT4/edit?usp=sharing">slide deck</a>
  <br>
  <a href="https://repository.library.brown.edu/studio/item/bdr:1140600/">poster</a>
   <br>
</div>

# Overview 

## Project Abstract

Many web applications today are read-heavy, meaning users are much more likely to access information from a server than they are to add new data. This read access is often a queried by the client over an internet connection. Computation to produce the desired read data, often on tables in a relational database, can be costly, and queries that are repeated are especially taxing for systems. Several alternative systems to relational databases exist, including Noria, a streaming dataflow system that stores data in a graph. Data and updates flow through the graph’s operator nodes, essentially precomputing a query's desired information, and are stored in ready to access tables called Views. Although this approach significantly reduces read access speeds, it still fails to circumvent the internet latency that comes with queries and data sent back and forth over the internet. 

SnakeRiver is a system where that dataflow graph exists across both server and client, executing server computation and storing View data in the client itself. Reads are therefore done locally and are incredibly fast. In specific graph scenarios, user-specific processing can also be done in the client, reducing server load and costs. This is achieved with a library of Rust functions that compiles into Web Assembly, allowing the creation of a graph that manipulates data in the browser. In our testing of several read and write scenarios, write speeds were reasonable for expected workloads, while reads remained very fast. Latency per operation was as much 10x faster than a Noria system under comparable workloads, showing that the system succeeds in lowering both latency and server computation in web applications that feature read heavy workloads.

## About

This repository contains creates the "graph server" that communicates with the client, currently locally. 

## Code Structure

- `src`: graph functions, nearly identical to clientside library aside from some changes in building dataflow graph and Root/Leaf nodes, which are now both stateless, 
Leaf nodes store websocket connections
- `main.rs`: server

## 🚴 Usage

In root directory to start up server:
```
cargo run
```

# Project Writeup

## Client vs Server Testing

