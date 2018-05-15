
extern crate fatcat;
extern crate diesel;

use self::fatcat::*;
use self::models::*;
use self::diesel::prelude::*;

fn main() {
    use diesel_demo::database_schema::creators::dsl::*;

    let connection = establish_connection();
    let results = creators.filter(published.eq(true))
        .limit(5)
        .load::<CreatorRev>(&connection)
        .expect("Error loading creators");

    println!("Displaying {} creators", results.len());
    for creator in results {
        println!("{}", creator.title);
        println!("----------\n");
        println!("{}", creator.body);
    }
}
