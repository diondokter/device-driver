# Cfg

Pretty much anywhere you can put docs/description, you can also put some cfg. They use the same syntax as the inside of the cfg attribute, e.g. `feature = "blah"`.

> [!IMPORTANT]
> The cfg's have no impact on the code generation other than forwarding the cfg's as attributes on items.

This presents a couple of challenges:

- It's quite hard to check whether a driver can compile with any combination of cfg's.
- The cfg's are resolved after code generation, so the toolkit can't check anything.
- It's hard to predict how the cfg attributes on various items interact.

So what does this all mean?

> [!CAUTION]
> - The support for cfg's are best effort only. Expect things to be weird or something to work against you.
> - Some analysis may not be done on objects with cfg which can lead to weird errors in the generated code since problems are not caught beforehand.

> [!WARNING]
> - If you use cfg's, check the generated code to see if everything looks alright.
> - Use cfg's only sparingly.
> - Test all realistic cfg combinations, preferrably even in CI.

If there is a problem and the toolkit can do better, then please make an issue!
