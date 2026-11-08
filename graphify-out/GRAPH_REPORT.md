# Graph Report - INVOICE  (2026-11-08)

## Corpus Check
- Corpus is ~1,790 words - fits in a single context window. You may not need a graph.

## Summary
- 21 nodes · 27 edges · 5 communities (2 shown, 3 thin omitted)
- Extraction: 78% EXTRACTED · 22% INFERRED · 0% AMBIGUOUS · INFERRED: 6 edges (avg confidence: 0.92)
- Token cost: 33,929 input · 0 output

## Community Hubs (Navigation)
- Invoice Integrity
- MVP Workflow
- Application Foundation
- Offline Desktop

## God Nodes (most connected - your core abstractions)
1. `Project Plan — Dual Zone Garage Invoice Generator` - 11 edges
2. `AGENTS.md — Working Rules` - 8 edges
3. `Prompt 01 — Application Foundation` - 5 edges
4. `Local-Only Offline Data Principle` - 2 edges
5. `Precise Monetary Calculations and Grosz Storage` - 2 edges
6. `Issued Invoice Snapshot and Immutability` - 2 edges
7. `Atomic Invoice Issuance and Number Assignment` - 2 edges
8. `Local Windows Desktop Application` - 2 edges
9. `Invoice Generator MVP Workflow` - 2 edges
10. `Local SQLite Data Persistence` - 2 edges

## Surprising Connections (you probably didn't know these)
- `Local-Only Offline Data Principle` --semantically_similar_to--> `Local Windows Desktop Application`  [INFERRED] [semantically similar]
  AGENTS.md → PROJECT_PLAN.md
- `Precise Monetary Calculations and Grosz Storage` --semantically_similar_to--> `Precise Financial Arithmetic`  [INFERRED] [semantically similar]
  AGENTS.md → PROJECT_PLAN.md
- `Issued Invoice Snapshot and Immutability` --semantically_similar_to--> `Issued Invoice Data Snapshots`  [INFERRED] [semantically similar]
  AGENTS.md → PROJECT_PLAN.md
- `Atomic Invoice Issuance and Number Assignment` --semantically_similar_to--> `Atomic Configurable Invoice Numbering`  [INFERRED] [semantically similar]
  AGENTS.md → PROJECT_PLAN.md
- `Tauri 2, React, TypeScript, and Vite Stack` --semantically_similar_to--> `Tauri 2, React, TypeScript, and Vite Bootstrap`  [INFERRED] [semantically similar]
  PROJECT_PLAN.md → PROMPT_01_BOOTSTRAP.md

## Hyperedges (group relationships)
- **Invoice Data Integrity and Correctness Principles** — project_plan_invoice_snapshots, project_plan_atomic_numbering, project_plan_precise_financial_arithmetic, project_plan_configurable_tax_settings [EXTRACTED 1.00]

## Communities (5 total, 3 thin omitted)

### Community 1 - "MVP Workflow"
Cohesion: 0.50
Nodes (4): Project Plan — Dual Zone Garage Invoice Generator, Local SQLite Data Persistence, MVP Scope Exclusions, Invoice Generator MVP Workflow

### Community 2 - "Application Foundation"
Cohesion: 0.40
Nodes (5): Tauri 2, React, TypeScript, and Vite Stack, Prompt 01 — Application Foundation, Bootstrap Foundation Scope, Polish Dual Zone Garage Interface, Tauri 2, React, TypeScript, and Vite Bootstrap

## Knowledge Gaps
- **3 isolated node(s):** `MVP Scope Exclusions`, `Bootstrap Foundation Scope`, `Polish Dual Zone Garage Interface`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 6 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Project Plan — Dual Zone Garage Invoice Generator` connect `MVP Workflow` to `Invoice Integrity`, `Application Foundation`, `Invoice Numbering`, `Offline Desktop`?**
  _High betweenness centrality (0.618) - this node is a cross-community bridge._
- **What connects `MVP Scope Exclusions`, `Bootstrap Foundation Scope`, `Polish Dual Zone Garage Interface` to the rest of the system?**
  _3 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Why does `AGENTS.md — Working Rules` connect `Invoice Integrity` to `MVP Workflow`, `Application Foundation`, `Invoice Numbering`, `Offline Desktop`?**
  _High betweenness centrality (0.416) - this node is a cross-community bridge._
- **Why does `Prompt 01 — Application Foundation` connect `Application Foundation` to `Invoice Integrity`, `MVP Workflow`?**
  _High betweenness centrality (0.255) - this node is a cross-community bridge._