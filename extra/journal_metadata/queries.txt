
    SELECT COUNT(DISTINCT issnl) FROM homepage WHERE domain = 'archive.org';

Top publishers that have journals in wayback:

    SELECT publisher, COUNT(*) FROM journal LEFT JOIN homepage ON journal.issnl = homepage.issnl WHERE homepage.domain = 'archive.org' GROUP BY journal.publisher ORDER BY COUNT(*) DESC LIMIT 10;

Top publishers in general:

    SELECT publisher, COUNT(*) from journal GROUP BY publisher ORDER BY COUNT(*) DESC LIMIT 25;

Homepage URL counts:

    SELECT COUNT(*) FROM homepage;
    SELECT COUNT(DISTINCT issnl) FROM homepage;
    SELECT issnl, COUNT(*) from homepage GROUP BY issnl ORDER BY COUNT(*) DESC LIMIT 10;

Top/redundant URLs and SURTs:

    SELECT surt, COUNT(*) FROM homepage GROUP BY surt ORDER BY COUNT(*) DESC LIMIT 10;

    SELECT publisher, name FROM journal LEFT JOIN homepage ON journal.issnl = homepage.issnl WHERE homepage.surt = 'com,benjamins)/';

fulltext coverage by publisher type:

    select publisher_type, avg(ia_frac), avg(preserved_frac), count(*) as journal_count, sum(release_count) as paper_count from journal group by publisher_type order by sum(release_count) desc;



    select publisher_type, avg(ia_frac), avg(preserved_frac), count(*) as journal_count, sum(release_count) as paper_count from journal group by publisher_type order by sum(release_count) desc;


    select publisher, count(*) as journal_count, avg(ia_frac) from journal where ia_frac < 0.05 group by publisher order by count(*) desc limit 10;


    select country, count(*) from journal group by country order by count(*) desc limit 10;
    select country, count(*), sum(release_count) from journal group by country order by sum(release_count) desc limit 10;

    select lang, count(*) from journal group by lang order by count(*) desc limit 10;
    select lang, count(*), sum(release_count) from journal group by lang order by sum(release_count) desc limit 10;

Coverage by sherpa color:

    select sherpa_color, sum(ia_count) from journal group by sherpa_color;

Blocked domains:

    select domain, count(*), sum(blocked) from homepage group by domain order by sum(blocked) desc limit 20;

Top duplicated domains:

    select url, count(*) from homepage group by url order by count(*) desc limit 20;
