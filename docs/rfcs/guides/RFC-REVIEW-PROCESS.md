# RFC Review Process

## How RFCs Work

1. **Proposal**: Author creates a draft PR with the RFC
2. **Discussion**: Team reviews, asks questions, suggests changes
3. **Revision**: Author addresses feedback
4. **Approval**: At least 2 core team members approve
5. **Merge**: RFC is merged to `main`
6. **Implementation**: Development follows the approved design

## RFC Template

```markdown
# A-XX: Title

**Status:** Proposed | Accepted | Rejected | Superseded
**Author:** Name
**Created:** YYYY-MM-DD
**Supersedes:** A-XX (if applicable)

---

## Summary
One paragraph description.

## Motivation
Why this is needed.

## Detailed Design
The technical design.

## Alternatives Considered
Other approaches and why they were rejected.

## Drawbacks
Known limitations.

## Unresolved Questions
Open questions for discussion.
```

## Review Criteria

1. **Completeness**: All sections filled out
2. **Consistency**: Aligns with other RFCs and master plan
3. **Feasibility**: Can be implemented in the target phase
4. **Trade-offs**: Alternatives considered, drawbacks acknowledged
5. **Clarity**: Design is unambiguous

## Approval Requirements

- 2 approvals from core team
- No unresolved objections
- All questions answered or deferred with rationale
