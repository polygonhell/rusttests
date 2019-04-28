# Thoughts


## Scan vs random access

Really scan vs join.

## In memory vs On disk
Need not match, but minor DB changes should result in small on disk changes.

### Add/Insert
### Delete
### Update



## Dictionary representation
Must map
  * Value &rarr; ID 
  * ID &rarr; Value

Fundamentally it's just a list of values, ID is the array index, but it's better if ID is invariate with the addition of new values, so either we scan for value, or we maintain an index of Value &rarr; ID

**For variably sized values**<br>
List Ptr &rarr; value<br>
Ordered list of indices into above<br>
Values

**For fixed size values**<br>
List of Values<br>
Ordered list of indices into above<br>

Note updates to the ordered list are likely to be monolithic, an colum with 1M unique values would require updating ~20E6 bit or ~2.5 MB's of index. on average it's half that but still unpalatible.

One alternative is to use a page based index that generally doesn't cause ripples through the entire structure on insert, possibly a B* tree or PMA, or some cache aware/oblivious structure with pointers beween pages, but this will havve a memory overhead and an increased cost per lookup.

PMA's oare of the order of 2-4x bigger than the index. BTrees will be < 2x




## Column vs Row storage

## Data representations for none fixed size fields
List of offsets, to actual data

## Managing deletes and updates

### Options
#### Insert Only
Allows for potential timetravel data retrieval.<br>
Pathological for some usecases.

#### Update in place


## Lock granularirty

## Write
insert vs update

Transactions take all locks required at commit and modify.
Transaction is "complete at the point" the commit is in the log.

Shared state will in effect be a table lock on write commit (updating the index).
Unless there is no "index".



## Write over Read
Write transaction commit begins after Read start, and modifies data that will be read during read transaction.
  * Since Write transaction could not have taken a lock on page already read in read transaction, read will complete as if write transaction had occurrred prior to the read transaction even if it started before.

## Read over Write
Not really an issue, write locks taken during commit process.

## Table snapshots (Copy on write)

Table is a view of a base Read Only table (the snapshot), and an overlay.

Could alloew multiple overlays with some form of merge operation.

If scans are ordered this is a simple merge operation by "Key". If scans are unordered requires scan of modifications, storing of keys and scan of parent with comparison to returned keys (i.e. requires an inmemory structure), could limit the size of this by forcing a complete copy from parent if modifications reach some threshold.


## Compression
Rely on compressed storage?

## Encryption
Rely on encrypted storage?

## Multiple instances



## Text indexing

  * Word -> ID
  * ID -> Doc Array
  * /Doc -> WID -> tok offset
  * /Doc -> tok array

// Need to do analysis assuming 1% unique words <br>
Document with 100K works - say 1K unique words/doc 20K overall<br>
20000 * (48 bits per word + 16 bits / ID) = 1,280,000 bits<br>
-- someway to compute Id's from words?

1000 Documents<br>
1000 * (10 bits per doc ID) * 1000 = 10,000,000 bits

Doc -> ID+Token<br>
words occur on average 100 times per doc<br>
1000 * (16 bits + 17 bits token offset * 100)*1000 = 1,716,000,000<br>

Doc -> tokenArray<br>
1000 * 100,000 * 16bits = 1,600,000,000<br>
-- could be extended to allow reconstruction of the document

Dominated by last 2 tables<br>

| Bits |
|---:|
| 1,716,000,000 |
| 1,600,000,000 |
|  10,000,000 |
|  1,280,000 |

|  |
|---:|
| 3,327,280,000 |

Total size = 415,910,000 Bytes<br>
Total Document size = 100,000 * 6 * 1000 = 600,000,000 Bytes

I would assume closer to 1:1 ratio given miscelaneous overhead.

Token array is not strictly necessary, it speeds up NGram resolution, without it.

| Bits |
|---:|
| 1,716,000,000 |
|  10,000,000 |
|  1,280,000 |

|  |
|---:|
| 1,727,280,000 |

Total size = 215,910,000 Bytes<br>

Tending towards about 1/2 of document size.








