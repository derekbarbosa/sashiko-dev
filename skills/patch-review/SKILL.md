# sashiko-patch-review Skill

This skill implements the 10-stage Sashiko review pipeline for analyzing Linux kernel patches. Use this skill to perform deep, multi-stage analysis of a patch to identify architectural flaws, logic errors, resource management issues, locking bugs, security vulnerabilities, and hardware-specific problems.

## 10-Stage Pipeline Overview

The review process follows the exact Sashiko server pipeline:

1.  **Stage 1: Analyze commit main goal**: Evaluates high-level intent, architectural flaws, and design correctness.
2.  **Stage 2: High-level implementation verification**: Checks if the code matches the commit message claims and handles corner cases.
3.  **Stage 3: Execution flow verification**: Exhaustive trace of control flow, logic branches, and error paths.
4.  **Stage 4: Resource management**: Audits for memory leaks, UAF, double frees, and lifecycle imbalances.
5.  **Stage 5: Locking and synchronization**: Deep dive into concurrency bugs, race conditions, and deadlocks.
6.  **Stage 6: Security audit**: Scrutinizes for overflows, TOCTOU, information leaks, and privilege escalation.
7.  **Stage 7: Hardware engineer's review**: (If applicable) Reviews register access, DMA, IRQs, and hardware state machines.
8.  **Stage 8: Deduplication and Consolidation**: Merges overlapping concerns from previous stages into a unique list.
9.  **Stage 9: Verification and severity estimation**: Validates concerns against the codebase, filters false positives, and assigns severity.
10. **Stage 10: LKML-friendly report generation**: Formats findings into a professional, inline-commented LKML email reply.

## Usage Instructions

When asked to review a patch using this skill:

1.  **Context Loading**:
    - Load `subsystem/subsystem.md` to identify relevant subsystem guides.
    - Load the verbatim instructions for each stage from `references/stage-instructions.md`.
    - Load global guidance files as specified in `stage-instructions.md` (e.g., `technical-patterns.md`, `locking.md`).

2.  **Step-by-Step Execution**:
    - Explicitly execute Stages 1 through 10 in sequence.
    - For Stages 1-7, use `grep_search` and `read_file` to perform the analysis. Document intermediate findings for each stage.
    - For Stage 8, consolidate the findings from all previous stages.
    - For Stage 9, verify each finding. You MUST look at the source code to prove or disprove a concern.
    - For Stage 10, generate the final LKML report.

## Mandatory Rules

- **Parity**: You MUST follow the instructions in `references/stage-instructions.md` for each stage to ensure parity with the Sashiko server.
- **No Markdown in Stage 10**: Final report must be plain text. Use `>` for quotes. No backticks.
- **Commit Header**: The report MUST start with `Commit: <hash>`, `Author: <name>`, `Subject: <subject>`.
- **Pre-existing Issues**: Explicitly state if an issue was pre-existing (e.g., "This isn't a bug introduced by this patch, but...").
- **Evidence-Based Dismissal**: To discard a concern, you MUST find concrete code evidence proving it is invalid.
