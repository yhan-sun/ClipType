# ADR-0020: Keep initial macOS render-host classification sticky for a session

- Status: Accepted
- Date: 2026-09-05
- Supersedes: ADR-0018

## Context

VS Code and Monaco can destroy and recreate temporary Accessibility focus nodes
while normal keyboard input is in progress. The replacement node may have a new
opaque identity and can briefly omit the `AXWebArea` ancestry or web-only
supported-attribute names used to classify the original node. Requiring every
sample to repeat that classification incorrectly stops a session in the middle
of a word or after the first generated closing brace.

Weakening all target comparisons to process identity alone would be unsafe. A
real application or top-level-window switch must stop before the next action,
and native controls should retain exact focused-element comparison.

## Decision

The target policy is selected by the initial capture and remains fixed for the
session:

- An initial render-host capture requires a stable frontmost process and focused
  top-level window. Later focus-node replacement in that same process and
  window is accepted even when the replacement sample temporarily lacks the
  original render-host classification.
- An initial native-control capture continues to require the same opaque focused
  element. A later node that merely looks like a render host cannot weaken that
  promise.
- A process change, focused-window change, missing required window identity,
  capture failure, disappearance, or ambiguity stops safely before dispatch.

The adapter stores only process ID, opaque Accessibility identity hashes, and a
content-free classification bit. It does not request or retain window titles,
field values, selected text, document text, DOM identifier values, or class-list
values.

## Consequences

Monaco node churn inside one top-level window no longer looks like a target
switch. True process and window changes still fail closed immediately at the
next revalidation point.

A shared web render surface can contain multiple logical fields that expose the
same process/window evidence. ClipType therefore does not claim exact logical
field or caret identity within a single render-host window. Physical
Accessibility permission and real VS Code behavior remain separate evidence
gates and cannot be inferred from unit tests or hosted launch smoke.
