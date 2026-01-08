---
description: How to decompose a User Story into granular Tasks using the CLI
---

1. **Read the Story Context**
   Fetch the details of the parent User Story to understand requirements.
   ```bash
   task show <STORY_ID>
   ```

2. **Plan the Tasks (Generate Markdown)**
   Create a markdown file (e.g., `plan.md`) defining the child tasks. 
   Use ID `0` to indicate new items. Set the parent ID to the User Story ID.

   *Example `plan.md`:*
   ```markdown
   #### Task: Implement Login UI (#0)
   **State:** New | **Parent:** #<STORY_ID> | **Effort:** 2h
   Create the React component for the login form.
   
   #### Task: Integrate Auth API (#0)
   **State:** New | **Parent:** #<STORY_ID> | **Effort:** 3h
   Connect the form to the backend auth endpoint.
   ```

3. **Validate the Plan**
   Check for hierarchy or syntax errors before executing.
   ```bash
   task import plan.md --validate
   ```

4. **Execute Breakdown (Create Items)**
   Import the markdown to create the work items in DevOps.
   ```bash
   task import plan.md
   ```

5. **Schedule Work (Optional)**
   After import, use the Output or `task list` to find the new IDs, then schedule them.
   ```bash
   task calendar schedule <NEW_TASK_ID> --duration 120 --title "Login UI"
   task calendar schedule <OTHER_TASK_ID> --duration 180 --title "Auth API"
   ```
