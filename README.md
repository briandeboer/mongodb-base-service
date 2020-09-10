# MongoDB base service

This provides a simple wrapper of MongoDB to assist with creating/updating/etc.. especially when some things are embedded documents. Docs are still TBD but look at [graphql-mongodb-boilerplate](https://github.com/briandeboer/graphql-mongodb-boilerplate) for an example of how to use.

## Testing
If you are using snapshots to do tests, you'll likely want to fix SystemTime to a fixed number to prevent things like `date_modified` or `date_created` updates to differ between snapshots. Because mongodb-base-service automatically updates the objects with those times, it has been updated to allow for mocking time (in v0.5.1). To include it, you'll need to enable the "test" feature. To do so, in your other crate enable it in `dev-dependencies`.

```
[dev-dependencies]
mongodb-base-service = { version = "0.5.1", features = ["graphql", "test"] }
```

After that you can set the time to a specific number:

```rust
use mongodb-base-service::mock_time;

// fix time to Jan 1, 2020 so that snapshots always have the same date_created, etc...
mock_time::set_mock_time(SystemTime::UNIX_EPOCH + Duration::from_millis(1577836800000)); 
```

Or if you need to increase the time to verify that the `date_modified` has changed:

```rust
// increase time by 10 seconds
mock_time::increase_mock_time(10000);
```

If you want to reset the time to normal SystemTime, you can use:

```rust
// revert to normal SystemTime
mock_time::clear_mock_time();
```

### Note - deprecated from 0.2.x

The return from the insert methods (insert_one, insert_many and insert_embedded) all return ids instead of the full objects now. Please do a find after if you need the full object.