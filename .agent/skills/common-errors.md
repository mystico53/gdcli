# Common Godot Errors — Cause & Fix

Error message patterns from gdcli output, their likely causes, and fixes.

> [!NOTE]
> gdcli reports errors in the format `res://file.gd:LINE: message`. The patterns below match the message portion.

## Runtime Errors

| Error Pattern | Likely Cause | Fix |
|---|---|---|
| `Invalid get index 'X' (on base: 'Nil')` | Accessing a property on a null object. Usually a `$NodeName` that doesn't exist or hasn't entered the tree yet. | Check the node path. Use `@onready` for node references. Verify the node exists in the scene tree. |
| `Cannot call method 'X' on a null value` | Calling a method on a variable that is `null`. The object was never assigned or was freed. | Initialize the variable. Check if the node exists with `has_node()` before accessing. |
| `Invalid operands 'X' and 'Y' for operator 'Z'` | Type mismatch in an operation (e.g., adding a `String` to an `int`). | Use explicit type conversion: `str()`, `int()`, `float()`. Add type annotations to catch these at parse time. |
| `Identifier 'X' not declared in the current scope` | Typo in variable/function name, or the variable was declared in a different scope. | Check spelling. Ensure the variable is declared before use. Check scope (e.g., variable declared inside an `if` block). |
| `Signal 'X' is already connected to 'Y'` | `connect()` called multiple times for the same signal-handler pair. | Connect in `_ready()` only (not in `_process()`). Check `is_connected()` before connecting. |
| `Node not found: 'X'` | `get_node()` or `$X` path doesn't match any node in the tree. | Verify the node name and path. Names are case-sensitive. Check parent scene structure. |
| `Attempted to free a locked object` | Calling `free()` or `queue_free()` on a node during iteration or physics callback. | Use `queue_free()` instead of `free()`. Use `call_deferred("queue_free")` if in a physics callback. |
| `Can't change this state while flushing queries` | Modifying physics state (adding/removing bodies) during physics processing. | Defer the operation: `call_deferred("remove_child", node)`. |
| `Stack overflow` | Infinite recursion — a function calls itself endlessly. | Check for recursive calls. Add base case to recursive functions. |
| `Index X is out of bounds (size: Y)` | Array access with an invalid index. | Check `array.size()` before accessing. Use `array.get(i)` which returns `null` on invalid index. |

## Parse / Compile Errors

| Error Pattern | Likely Cause | Fix |
|---|---|---|
| `Unexpected "Indent" in class body` | Missing colon `:` after `func`, `if`, `for`, `while`, `match`, etc. | Add `:` at the end of the statement line. |
| `Expected ":" after function declaration` | Same as above — missing colon after `func name()`. | `func name() -> void:` (note the colon). |
| `Unexpected token` | Syntax error — invalid GDScript syntax. | Check for missing parentheses, brackets, or operators. |
| `The function signature doesn't match the parent` | Overriding a parent method with different parameters. | Match the parent's function signature exactly. |
| `Cannot find member 'X' in base 'Y'` | Accessing a property/method that doesn't exist on the type. | Check the Godot docs for the correct property name. Verify the node type. |
| `Could not find type 'X' in the current scope` | Using a class name that doesn't exist or isn't imported. | Add `class_name` to the script, or use `preload()` to load it. |
| `The method 'X' is not declared in the current class` | Calling a method that doesn't exist. | Check spelling. Verify the method is defined in the correct script. |
| `Too few arguments for 'X()' call` | Function called with fewer arguments than declared. | Check the function signature and pass all required arguments. |
| `Too many arguments for 'X()' call` | Function called with more arguments than declared. | Remove extra arguments or add parameters to the function. |
| `Value of type 'X' cannot be assigned to a variable of type 'Y'` | Type mismatch in assignment with static typing. | Cast the value or change the variable type. |

## Scene / Resource Errors

| Error Pattern | Likely Cause | Fix |
|---|---|---|
| `Failed to load resource 'res://X'` | Referenced file doesn't exist or has been moved. | Check the file path. Run `gdcli uid fix` to fix stale UID references. |
| `res://X: No such file or directory` | File referenced in a scene or script is missing. | Restore the file or update the reference. Run `gdcli scene validate` to find broken refs. |
| `Circular reference detected` | Scene A instances scene B which instances scene A. | Restructure the scene tree to break the cycle. |

## gdcli-Specific Messages

| Message | Meaning | Action |
|---|---|---|
| `project.godot not found in current directory` | gdcli was run outside a Godot project root. | `cd` to the directory containing `project.godot`. |
| `Process timed out after Xs` | Godot didn't exit within the timeout. For `run`, this means the script didn't call `quit()`. | Add `quit()` at the end of headless scripts. Increase `--timeout` for long-running operations. |
| `Could not find a Godot binary` | No Godot found via `GODOT_PATH`, PATH, or common locations. | Set `GODOT_PATH=C:\path\to\godot.console.exe`. |
