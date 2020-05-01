# MongoDB base service

This provides a simple wrapper of MongoDB to assist with creating/updating/etc.. especially when some things are embedded documents. Docs are still TBD but look at [graphql-mongodb-boilerplate](https://github.com/briandeboer/graphql-mongodb-boilerplate) for an example of how to use.

### Note - deprecated from 0.2.x
The return from the insert methods (insert_one, insert_many and insert_embedded) all return ids instead of the full objects now. Please do a find after if you need the full object.