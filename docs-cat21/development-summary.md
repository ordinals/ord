
# CAT-21 and LockTimeOrdinal Development Summary

## 1. What is CAT-21 and LockTimeOrdinal Protocols?

### CAT-21 Overview
- **CAT-21** is an fun protocol that extends Bitcoin Ordinals by leveraging the `nLockTime` field in Bitcoin transactions.
- It focuses on enabling new digital asset types by associating sats with transactions having specific `nLockTime` values.
- These assets, called **LockTimeOrdinals (LTOs)**, are sats tied to transactions with a defined `nLockTime` range.

### LockTimeOrdinal Rules
- LTOs are created for transactions with an `nLockTime` between `0` and a hardcoded upper limit (e.g., `10000`).
- Each transaction generates one LTO, tied to the **first sat of the first output**.
- Attributes of an LTO include:
  - Transaction ID
  - `nLockTime` value
  - Incremental number (per `nLockTime` value)
  - Block ID, height, and timestamp
  - Fee, size, and weight of the transaction
  - Value of the first output
  - The satoshi and address associated with the first output

### Purpose and Applications
- The first prototocl that uses the `nLockTime` field to create new asset types is CAT-21 with nLockTime set to 21.
- CAT-21 opens the door for new protocols and asset types. It's a proof of concept that shows how the `nLockTime` field can be used to create new asset types.

---

## 2. Notable changes to the `ord` Client (to be continued)

### New Entities and Structures
- **`LockTimeOrdinalEntry`**: Represents an indexed LTO
- **Database Table**: Added `LOCKTIME_ORDINAL_TABLE` to store `LockTimeOrdinalEntry` data.
- **Database Table**: Added `LOCK_TIME_TO_NUMBER is used to maintain a mapping between specific nLockTime values and their respective incrementing ordinal numbers.


### Description of the `LOCKTIME_ORDINAL_TABLE` table

Stores metadata for each LockTimeOrdinal (LTO) created, ensuring all relevant data about the LTO is persistently tracked and retrievable.

```rust
define_table! { LOCKTIME_ORDINAL_TABLE, &[u8; 32], LockTimeOrdinalEntryValue }


### Description of the `LOCK_TIME_TO_NUMBER` table

The `LOCK_TIME_TO_NUMBER` table is used to maintain a mapping between specific
`nLockTime` values and their respective incrementing ordinal numbers for LockTimeOrdinal (LTO) assets.
 This table is essential to ensure that each `nLockTime` value is tracked independently and has its own sequence of LTO numbers.

#### Purpose
- **Track Ordinal Numbers per `nLockTime`:** Assign and increment a unique ordinal number for each `nLockTime` value.
- **Persistence Across Runs:** Ensure that the numbering continues correctly even if the indexing process is restarted.
- **Efficient Lookups:** Quickly fetch the next number for a given `nLockTime` without recalculating or keeping data in memory.

#### Table Structure
```rust
define_table! { LOCK_TIME_TO_NUMBER, u32, u32 }
```

#### Fields
1. **Key (`u32`):**
   - Represents the `nLockTime` value.
   - Serves as the unique identifier for this mapping.
   - Example: `21` for `nLockTime=21`.

2. **Value (`u32`):**
   - Represents the next available ordinal number for the given `nLockTime`.
   - Starts from `0` and increments as new LockTimeOrdinals are created.

#### Usage
1. **Insert or Increment Logic:**
   - If a specific `nLockTime` exists in the table, the value is incremented to determine the next ordinal number.
   - If the `nLockTime` is not present, it is initialized with a value of `0`.

2. **Example Workflow:**
   - If `LOCK_TIME_TO_NUMBER` contains an entry `(21, 5)`, it means the next ordinal number for `nLockTime=21` is `5`.
   - When a new LockTimeOrdinal is created for `nLockTime=21`, the table is updated to `(21, 6)`.

3. **Querying:**
   - The table is queried whenever a transaction with a specific `nLockTime` is processed.
   - Ensures that each LockTimeOrdinal is assigned the correct ordinal number.

4. **Consistency:**
   - Since the table operates within a transaction (`WriteTransaction`), it ensures that updates are atomic and consistent.
