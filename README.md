# Cassandra Query Language (CQL) parser

Hand crafted Apache Cassandra CQL parser, implemented in Rust.

_WARNING: This library is still in very early stage, and APIs and AST will change._

## Design goals

The goal of this library is to provide standalone CQL parser that can be used in tools,
such as pretty printer, auto completion, schema visualizer, etc.

The design goals are:

- Minimum dependencies
- Detailed error reporting
- WebAssembly compatible
- Complete and validatable AST
- Supports both CQL to AST and AST to CQL
- Parsing multiple statements

## Supported CQL versions

- [CQL v3.4.3](https://github.com/apache/cassandra/blob/cassandra-4.0.0/doc/cql3/CQL.textile)

## Implemented statements

- [x] SELECT statement
    - SELECT JSON / DISTINCT is not yet implemented
- [x] INSERT statement
- [x] UPDATE statement
- [ ] BATCH statement
- [ ] DELETE statement
- [ ] USE statement
- [ ] TRUNCATE statement
- [x] CREATE KEYSPACE statement
- [x] CREATE TABLE statement
- [x] CREATE INDEX statement
- [ ] DROP KEYSPACE statement
- [ ] DROP TABLE statement
- [ ] DROP INDEX statement
- [ ] ALTER TABLE statement
- [ ] ALTER KEYSPACE statement
- [ ] GRANT PERMISSIONS statement
- [ ] REVOKE PERMISSIONS statement
- [ ] LIST PERMISSIONS statement
- [ ] CREATE USER statement (deprecated)
- [ ] ALTER USER statement (deprecatd)
- [ ] DROP USER statement (deprecated)
- [ ] LIST USERS statement (deprecated)
- [ ] CREATE TRIGGER statement
- [ ] DROP TRIGGER statement
- [x] CREATE TYPE statement
- [ ] ALTER TYPE statement
- [ ] DROP TYPE statement
- [ ] CREATE FUNCTION statement
- [ ] DROP FUNCTION statement
- [ ] CREATE AGGREGATE statement
- [ ] DROP AGGREGATE statement
- [ ] CREATE ROLE statement
- [ ] ALTER ROLE statement
- [ ] DROP ROLE statement
- [ ] LIST ROLES statement
- [ ] GRANT ROLE statement
- [ ] REVOKE ROLE statement
- [x] CREATE MATERIALIZED VIEW statement
- [ ] DROP MATERIALIZED VIEW statement
- [ ] ALTER MATERIALIZED VIEW statement
- [ ] DESCRIBE statement

## TODOs

- Binding variables are not yet supported.