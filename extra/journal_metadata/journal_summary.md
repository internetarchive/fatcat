
# Fatcat "Chocula" Journal Metadata Summary

This report is auto-generated from a sqlite database file, which should be available/included.

```sql
SELECT datetime('now');
```

Note that pretty much all of the fatcat release stats are on a *release*, not
*work* basis, so there may be over-counting. Also, as of July 2019 there were
over 1.5 million OA longtail releases which are *not* linked to a container
(journal).

### Basics

Top countries by journal count (and fatcat release counts):

```sql
SELECT country, COUNT(*) AS journal_count, sum(release_count) from journal group by country order by count(*) desc limit 10;
```

Top languages by journal count (and fatcat release counts):

```sql
SELECT lang, COUNT(*) as journal_count, sum(release_count) as release_count FROM journal GROUP BY lang ORDER BY COUNT(*) DESC LIMIT 10;
```

Aggregate fatcat fulltext release coverage by OA status:

```sql
SELECT is_oa, COUNT(*) AS journal_count, SUM(release_count), SUM(ia_count), ROUND(1. * SUM(ia_count) / SUM(release_count), 2) as total_ia_frac FROM journal GROUP BY is_oa;
```

### Publisher Segmentation

Big publishers by journal count:

```sql
SELECT publisher, COUNT(*) AS journal_count, SUM(release_count) from journal GROUP BY publisher ORDER BY COUNT(*) DESC LIMIT 15;
```

Number of publishers with 3 or fewer journals:

```sql
SELECT COUNT(*) FROM (SELECT publisher, COUNT(*) as journal_count FROM journal GROUP BY publisher) WHERE journal_count <= 3;
```

Fulltext coverage by publisher type:

```sql
SELECT publisher_type, ROUND(1.0 * SUM(ia_count) / SUM(release_count), 2) as ia_total_frac, ROUND(1.0 * SUM(preserved_count) / SUM(release_count), 2) as preserved_total_frac, count(*) as journal_count, sum(release_count) as paper_count from journal group by publisher_type order by sum(release_count) desc;
```

Fulltext coverage by publisher type (NOTE: averaging fractions without weighing by release count, intentionally):

```sql
SELECT publisher_type, ROUND(1.0 * AVG(ia_frac), 2) as avg_ia_frac, ROUND(1.0 * AVG(preserved_frac), 2) as avg_preserved_frac, count(*) as journal_count, sum(release_count) as paper_count from journal group by publisher_type order by sum(release_count) DESC;
```

Number of journals with no releases (metadata or fulltext) in fatcat:

```sql
SELECT publisher_type, COUNT(*) AS journals_with_no_releases FROM journal WHERE release_count = 0 GROUP BY publisher_type ORDER BY COUNT(*) DESC;
```

### IA Fulltext Coverage

Coverage by sherpa color:

```sql
SELECT sherpa_color, SUM(ia_count) as ia_fulltext_count, SUM(release_count) as release_count, ROUND(1.0 * SUM(ia_count) / SUM(release_count), 2) as total_ia_frac FROM journal GROUP BY sherpa_color;
```

Top publishers with very little IA coverage (NOTE: averaging fractions without weight by journal size):

```sql
SELECT publisher, count(*) as journal_count, ROUND(avg(ia_frac),3) from journal where ia_frac < 0.05 group by publisher order by count(*) desc limit 10;
```

### Homepages

Journal counts by homepage status:

```sql
SELECT any_homepage, any_live_homepage, any_gwb_homepage, COUNT(*), ROUND(1.0 * COUNT(*) / (SELECT COUNT(*) FROM journal), 2) AS frac FROM journal GROUP BY any_homepage, any_live_homepage, any_gwb_homepage;
```

Number of unique journals that have a homepage pointing to wayback or archive.org:

```sql
SELECT COUNT(DISTINCT issnl) FROM homepage WHERE domain = 'archive.org';
```

Top publishers that have journals in wayback:

```sql
SELECT publisher, COUNT(*) FROM journal LEFT JOIN homepage ON journal.issnl = homepage.issnl WHERE homepage.domain = 'archive.org' GROUP BY journal.publisher ORDER BY COUNT(*) DESC LIMIT 10;
```

Homepage URL counts:

```sql
SELECT COUNT(*) as rows, COUNT(DISTINCT issnl) as issnls, COUNT(DISTINCT surt) as surts FROM homepage;
```

Journals with most unique SURTs:

```sql
SELECT issnl, COUNT(*) from homepage GROUP BY issnl ORDER BY COUNT(*) DESC LIMIT 10;
```


Blocked domains:

```sql
SELECT domain, count(*), sum(blocked) from homepage group by domain order by sum(blocked) desc limit 20;
```

Top duplicated URLs and SURTs:

```sql
SELECT url, COUNT(*) FROM homepage GROUP BY url ORDER BY COUNT(*) DESC LIMIT 10;
```

Top terminal URLs catch cases where many URLs redirect to a single page:

```sql
SELECT terminal_url, COUNT(DISTINCT issnl) FROM homepage WHERE terminal_url IS NOT NULL GROUP BY terminal_url ORDER BY COUNT(DISTINCT issnl) DESC LIMIT 20;
```

```sql
SELECT surt, COUNT(*) FROM homepage GROUP BY surt ORDER BY COUNT(*) DESC LIMIT 10;
```
