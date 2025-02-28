# Atomic Load Balancer Forwarder for Chrome Instances

## Overview

This document outlines the architecture and design of a library that acts as a load balancer (LB) and forwarder to manage multiple headless Chrome instances on remote servers. This system addresses the security limitations posed by Chrome's restriction on remote connections and optimizes resource utilization across machines.

## Objectives

- Enable secure forwarding of requests to internal Chrome instances on remote servers.
- Improve resource utilization by distributing the load across multiple machines and CPU cores.
- Implement an efficient load balancing strategy using a round-robin approach to spawn multiple Chrome instances.

## Architecture

Below is a high level design for the LB.

### Forwarding

The primary function of the library is to forward all incoming requests to the appropriate Chrome instance. This allows for centralized management and control over the Chrome connections, enhancing security and flexibility.

### Performance

Chrome's multi-threading capabilities include certain limitations, particularly when scaling CPU usage beyond 100-200% on a single core, leading to performance degradation. To mitigate this, the load balancer is designed to handle workloads across multiple machines.

- The system will utilize a round-robin strategy to spawn and manage multiple Chrome instances, maximizing CPU core usage.
- Efficient management of Chrome instances ensures optimal resource allocation and performance.

## Design

The following steps outline the core design principles and implementation strategy:

1. **Atomic Tracker for Instance Management:**
   - Use the `/json/version` endpoint to maintain an atomic tracker that manages the spawning of new Chrome instances.
   - Track the last Process ID (PID) in an atomic manner to ensure that each instance is correctly managed and tracked.

2. **Proxy Forwarder Logic:**
   - Identify the current target instance and route the connection to the corresponding node using the atomic tracker from `/json/version`.
   - Decrement the atomic tracker upon request completion to maintain an accurate count of active instances.

3. **Index and Tracker Synchronization:**
   - Synchronize the proxy forwarder with the atomic tracker and current index values based on `/json/version`.
   - Ensure consistency between the tracker and active instances.

4. **Connection Limit Management:**
   - Implement a maximum connection limit for atomic trackers to facilitate the spawning of new Chrome instances.
   - Prioritize reusing available slots before initiating additional instances on other machines.

5. **PID Tracking:**
   - Use `/json/version` trackers to maintain a record of active PIDs, enabling efficient cleanup and management of instance positions.

6. **Modify modify_json_output**
   - Make the changes for `modify_json_output` so it returns the proper port binding.

This architecture and design aim to optimize performance, security, and resource management for handling multiple Chrome instances, ensuring scalable and efficient operations.