# Architecture

```text
Request -> Service::create validation -> in-memory BTreeMap -> returned identifier
                                      \\-> Service::get -> optional body
```

`Service` owns all process-local state. The interface is synchronous; validation errors are returned
to the caller. IDs are trimmed and unique for the lifetime of one `Service`. There is no persistence,
remote call, retry, authentication, shutdown protocol, or concurrent access contract. Changing any of
those boundaries requires a new intent/decision and additional obligations rather than treating this
example as a production blueprint.
