# Move components to the bottom

- STATUS: OPEN
- PRIORITY: 120
- TAGS: v0.1.5


Currently, components must be defined inside an endpoint description. However, a better approach would be to support **shared components** that are defined independently and are not tied to a specific endpoint.

We should introduce two types of components:

### 1. General (Common) Components
- Defined separately from endpoints.
- Can be referenced by multiple endpoints or other components.
- Must be referenced at least once somewhere in the project.
- **Validation error:** If a General component is not referenced anywhere, report an error.

### 2. Internal Components
- Defined inside a specific endpoint.
- Can only be used within the endpoint where they are defined.
- Cannot be referenced by other endpoints or external components.
- **Validation error:** If an Internal component is referenced outside its endpoint, report an error and suggest either:
  - Moving the component to General Components.
  - Removing the invalid reference.

### Summary
The main goal is to clearly separate **reusable components (General)** from **endpoint-specific components (Internal)** and enforce their usage rules through validation.
