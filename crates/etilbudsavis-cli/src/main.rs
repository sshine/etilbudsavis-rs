//! `etilbudsavis` — a command-line client for the app's backend.

use std::io::Write as _;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use etilbudsavis_api::{
    Business, Client, CreatedShoppingList, Location, OffersQuery, Pagination, PersonUpdate,
    Session, ShoppingListItemDraft, ShoppingListItemUpdate, StoredSession, StoresQuery, timestamp,
};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Query eTilbudsavis offers, catalogues, stores, and lists"
)]
struct Args {
    /// Emit JSON instead of the short human-readable listing.
    #[arg(long, global = true, env = "ETILBUDSVIS_JSON")]
    json: bool,

    /// Account token. Defaults to the token file and then ETILBUDSVIS_TOKEN.
    #[arg(long, global = true, env = "ETILBUDSVIS_TOKEN")]
    token: Option<String>,

    /// Where the account token is stored.
    #[arg(long, global = true, env = "ETILBUDSVIS_SESSION_FILE")]
    session_file: Option<PathBuf>,

    /// Backend base URL, for staging or tests.
    #[arg(long, global = true, env = "ETILBUDSVIS_BASE_URL", hide = true)]
    base_url: Option<String>,

    /// Android API key override.
    #[arg(long, global = true, env = "ETILBUDSVIS_API_KEY", hide = true)]
    api_key: Option<String>,

    /// Seconds to wait for one request.
    #[arg(long, global = true, default_value_t = 30)]
    timeout: u64,

    /// Latitude for location-aware results.
    #[arg(long, global = true)]
    latitude: Option<f64>,

    /// Longitude for location-aware results.
    #[arg(long, global = true)]
    longitude: Option<f64>,

    /// Country code used with --latitude/--longitude.
    #[arg(long, global = true, default_value = "DK")]
    country: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Account status, sign-in, consent, and sign-out.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },

    /// Catalogue groups, fronts, and publications.
    Catalogue {
        #[command(subcommand)]
        command: CatalogueCommand,
    },

    /// Businesses.
    Business {
        #[command(subcommand)]
        command: BusinessCommand,
    },

    /// Offers.
    Offer {
        #[command(subcommand)]
        command: OfferCommand,
    },

    /// Physical stores.
    Store {
        #[command(subcommand)]
        command: StoreCommand,
    },

    /// Search suggestions and SEO landing pages.
    Search {
        #[command(subcommand)]
        command: SearchCommand,
    },

    /// Videos and recipe ingredients.
    Video {
        #[command(subcommand)]
        command: VideoCommand,
    },

    /// Favourite businesses.
    Favorites {
        #[command(subcommand)]
        command: FavoritesCommand,
    },

    /// Shopping lists.
    Lists {
        #[command(subcommand)]
        command: ListCommand,
    },

    /// Saved offer searches.
    Agents {
        #[command(subcommand)]
        command: AgentCommand,
    },

    /// The service's location and reverse geocoding.
    Location {
        #[command(subcommand)]
        command: LocationCommand,
    },

    /// The personalised "for you" row.
    ForYou,

    /// The privacy policy as Markdown.
    Privacy {
        /// Locale, such as `da` or `en`.
        #[arg(long, default_value = "da")]
        locale: String,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Report whether a token is stored and who it belongs to.
    Status,
    /// Create an anonymous account and store its token.
    Anonymous,
    /// Start an email one-time-code login.
    Login {
        /// Email address to send the code to.
        email: String,
    },
    /// Finish a login with an OTP id and the mailed code.
    Verify {
        /// The `otpId` from `auth login`.
        otp_id: String,
        /// The code from the email.
        code: String,
    },
    /// Forget the local token.
    Logout,
    /// Accept or reject the privacy policy.
    Consent {
        /// `accepted` or `rejected`.
        state: String,
    },
    /// Replace the enabled notification kinds.
    Notifications {
        /// Kinds such as `offer_agent_new_offers_v2`.
        kinds: Vec<String>,
    },
    /// Update the account country.
    Country {
        /// ISO 3166-1 alpha-2 code.
        code: String,
    },
}

#[derive(Debug, Subcommand)]
enum CatalogueCommand {
    /// The home screen's category groups and their fronts.
    List,
    /// Fronts filtered by business, category, or activity.
    Fronts {
        /// Business slug.
        #[arg(long)]
        business: Option<String>,
        /// Category slug, such as `groceries`.
        #[arg(long)]
        category: Option<String>,
        /// Hide businesses without an active publication.
        #[arg(long)]
        hide_inactive: bool,
    },
    /// One publication's metadata.
    Publication {
        /// Publication id.
        id: String,
    },
    /// Other publications from a business.
    RelatedPublications {
        /// Business public id.
        business_id: String,
        /// Optional location id.
        #[arg(long)]
        location_id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum BusinessCommand {
    /// All businesses, optionally filtered by category.
    List {
        /// Category slug.
        #[arg(long)]
        category: Option<String>,
        /// Maximum rows.
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// One business by slug.
    Show {
        /// Business slug.
        slug: String,
    },
    /// The long marketing description.
    Description {
        /// Business public id.
        public_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum OfferCommand {
    /// A page of offers.
    List {
        /// Business slug or public id.
        #[arg(long)]
        business: Option<String>,
        /// Free-text search.
        #[arg(long)]
        search: Option<String>,
        /// Department slug.
        #[arg(long)]
        department: Option<String>,
        /// Sort order, such as `popularity_desc`.
        #[arg(long)]
        sort: Option<String>,
        /// Hide offers that have not started.
        #[arg(long)]
        hide_upcoming: bool,
        /// Rows to return.
        #[arg(long, default_value_t = 30)]
        limit: u32,
        /// Follow pagination up to --limit rows.
        #[arg(long)]
        all: bool,
    },
    /// One offer.
    Show {
        /// Offer public id.
        public_id: String,
    },
    /// Product variants behind an offer.
    Products {
        /// Offer public id.
        public_id: String,
    },
    /// Offers related to an offer.
    Related {
        /// Offer public id.
        public_id: String,
    },
    /// Offers grouped by department.
    Departments {
        /// Business public id.
        business_id: String,
        /// Maximum departments.
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Debug, Subcommand)]
enum StoreCommand {
    /// Stores filtered by business, offer, or text.
    List {
        /// Business slug or public id.
        #[arg(long)]
        business: Option<String>,
        /// Publication id.
        #[arg(long)]
        publication: Option<String>,
        /// Offer public id.
        #[arg(long)]
        offer: Option<String>,
        /// Free-text store search.
        #[arg(long)]
        search: Option<String>,
        /// Rows to return.
        #[arg(long, default_value_t = 20)]
        limit: u32,
        /// Follow pagination up to --limit rows.
        #[arg(long)]
        all: bool,
    },
    /// One store.
    Show {
        /// Store id.
        id: String,
    },
}

#[derive(Debug, Subcommand)]
enum SearchCommand {
    /// Popular search terms.
    Top {
        /// Rows to return.
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Suggestions for a phrase.
    Suggestions {
        /// The phrase typed so far.
        query: String,
    },
    /// The business whose name best matches a phrase.
    Business {
        /// The phrase.
        term: String,
    },
    /// SEO landing-page content for a phrase.
    Seo {
        /// The phrase.
        term: String,
    },
    /// Offers matching a phrase.
    Offers {
        /// The phrase.
        query: String,
        /// Rows to return.
        #[arg(long, default_value_t = 30)]
        limit: u32,
    },
}

#[derive(Debug, Subcommand)]
enum VideoCommand {
    /// The video feed.
    List {
        /// Rows to return.
        #[arg(long, default_value_t = 20)]
        limit: u32,
        /// Free-text search.
        #[arg(long)]
        search: Option<String>,
    },
    /// One video.
    Show {
        /// Video id.
        id: String,
    },
    /// Recipe ingredients for a video.
    Ingredients {
        /// Video id.
        id: String,
    },
    /// Videos the account liked.
    Liked,
    /// Videos the account uploaded.
    Mine,
    /// Like a video.
    Like {
        /// Video id.
        id: String,
    },
    /// Remove a like.
    Unlike {
        /// Video id.
        id: String,
    },
}

#[derive(Debug, Subcommand)]
enum FavoritesCommand {
    /// Favourite businesses.
    List,
    /// Add a business by slug or public id.
    Add {
        /// Business slug or public id.
        business: String,
    },
    /// Remove a business by slug or public id.
    Remove {
        /// Business slug or public id.
        business: String,
    },
}

#[derive(Debug, Subcommand)]
enum ListCommand {
    /// Lists owned by or shared with the account.
    List,
    /// Items on a list.
    Show {
        /// List id.
        id: i64,
    },
    /// Create a list.
    Create {
        /// List name.
        name: String,
    },
    /// Rename a list.
    Rename {
        /// List id.
        id: i64,
        /// New name.
        name: String,
    },
    /// Delete a list for everyone.
    Delete {
        /// List id.
        id: i64,
    },
    /// Leave a shared list.
    Leave {
        /// List id.
        id: i64,
    },
    /// Add a free-text item.
    Add {
        /// List id.
        id: i64,
        /// Item name.
        name: String,
        /// Quantity.
        #[arg(long, default_value_t = 1)]
        count: i64,
        /// Comment or pack description.
        #[arg(long)]
        comment: Option<String>,
        /// Estimated price.
        #[arg(long)]
        price: Option<f64>,
    },
    /// Update an item by its client id.
    Update {
        /// List id.
        id: i64,
        /// Item client id.
        item: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New quantity.
        #[arg(long)]
        count: Option<i64>,
        /// Tick or untick the item.
        #[arg(long)]
        ticked: Option<bool>,
        /// New comment.
        #[arg(long)]
        comment: Option<String>,
        /// New estimated price.
        #[arg(long)]
        price: Option<f64>,
    },
    /// Remove an item by its client id.
    Remove {
        /// List id.
        id: i64,
        /// Item client id.
        item: String,
    },
    /// Remove every item on a list.
    Clear {
        /// List id.
        id: i64,
    },
    /// Accept a share link.
    Accept {
        /// The share token from the link.
        share_token: String,
    },
    /// Ask the service to choose a department for an item.
    AutoCategorize {
        /// List id.
        id: i64,
        /// Item client id.
        item: String,
    },
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    /// Saved searches.
    List,
    /// Create a saved search.
    Create {
        /// Search term.
        term: String,
        /// Geohash to bind the search to.
        #[arg(long, default_value = "u3buy")]
        geohash: String,
        /// Maximum radius in metres.
        #[arg(long)]
        max_radius: Option<u64>,
        /// Include offers that have not started.
        #[arg(long)]
        include_upcoming: bool,
    },
    /// Delete a saved search.
    Delete {
        /// Public id.
        public_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum LocationCommand {
    /// The service's fallback location.
    Show,
    /// Reverse-geocode a point.
    Lookup {
        /// Latitude.
        latitude: f64,
        /// Longitude.
        longitude: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match run(&args).await {
        Ok(()) => Ok(()),
        Err(error) => {
            if !args.json {
                eprintln!("etilbudsavis: {error:#}");
            }
            Err(error)
        }
    }
}

async fn run(args: &Args) -> Result<()> {
    match &args.command {
        Command::Auth { command } => auth(args, command).await,
        Command::Catalogue { command } => catalogue(args, command).await,
        Command::Business { command } => business(args, command).await,
        Command::Offer { command } => offer(args, command).await,
        Command::Store { command } => store(args, command).await,
        Command::Search { command } => search(args, command).await,
        Command::Video { command } => video(args, command).await,
        Command::Favorites { command } => favorites(args, command).await,
        Command::Lists { command } => lists(args, command).await,
        Command::Agents { command } => agents(args, command).await,
        Command::Location { command } => location(args, command).await,
        Command::ForYou => {
            let client = args.client().await?;
            let row = client.for_you().await?;
            args.output(&row)
        }
        Command::Privacy { locale } => {
            let client = args.client().await?;
            let text: String = client
                .data("privacyPolicy", serde_json::json!({ "locale": locale }))
                .await?;
            if args.json {
                args.output(&serde_json::json!({ "locale": locale, "markdown": text }))
            } else {
                println!("{text}");
                Ok(())
            }
        }
    }
}

impl Args {
    fn session(&self) -> Session {
        Session::at(
            self.session_file
                .clone()
                .unwrap_or_else(Session::default_path),
        )
    }

    fn stored_token(&self) -> Option<String> {
        if let Some(token) = &self.token {
            return Some(token.clone());
        }
        self.session()
            .load()
            .ok()
            .flatten()
            .map(|session| session.token.expose().to_owned())
    }

    async fn client(&self) -> Result<Client> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .api_key(
                self.api_key
                    .clone()
                    .unwrap_or_else(|| etilbudsavis_api::DEFAULT_API_KEY.to_owned()),
            );
        if let Some(base_url) = &self.base_url {
            builder = builder.base_url(base_url);
        }
        if let Some(token) = self.stored_token() {
            builder = builder.token(token);
        }
        if let (Some(latitude), Some(longitude)) = (self.latitude, self.longitude) {
            let probe = builder.clone().build()?;
            let info = probe.location_info(latitude, longitude).await?;
            let location = Location {
                latitude,
                longitude,
                geohash: info.geohash,
                city: None,
                country: Some(info.country.unwrap_or_else(|| self.country.clone())),
                mode: Some("manual".to_owned()),
            };
            builder = builder.location(location);
        }
        builder.build().map_err(Into::into)
    }

    /// A client with an account, creating an anonymous one if needed.
    async fn account(&self) -> Result<Client> {
        if self.stored_token().is_some() {
            return self.client().await;
        }
        let client = self.client().await?;
        let session = client.create_anonymous().await?;
        self.session().save(&StoredSession {
            token: session.token,
            email: session.email,
            created_at: Utc::now(),
        })?;
        eprintln!(
            "created an anonymous account in {}",
            self.session().path().display()
        );
        self.client().await
    }

    fn output<T: serde::Serialize>(&self, value: &T) -> Result<()> {
        println!("{}", serde_json::to_string_pretty(value)?);
        Ok(())
    }
}

fn format_time(time: Option<DateTime<Utc>>) -> String {
    time.map(|time| time.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_owned())
}

fn print_business(business: &Business) {
    println!(
        "{}\t{}\t{}\t{}",
        business.public_id,
        business.name,
        business.primary_category_id.as_deref().unwrap_or("-"),
        business.slug().unwrap_or("-")
    );
}

async fn resolve_business_id(client: &Client, value: &str) -> Result<String> {
    match client.business(value).await {
        Ok(business) => Ok(business.public_id),
        Err(_) => Ok(value.to_owned()),
    }
}

async fn auth(args: &Args, command: &AuthCommand) -> Result<()> {
    match command {
        AuthCommand::Status => {
            let Some(token) = args.stored_token() else {
                if args.json {
                    return args.output(&serde_json::json!({ "authenticated": false }));
                }
                println!(
                    "signed out (no token at {})",
                    args.session().path().display()
                );
                return Ok(());
            };
            let client = Client::builder()
                .token(token)
                .base_url(
                    args.base_url
                        .clone()
                        .unwrap_or_else(|| etilbudsavis_api::DEFAULT_BASE_URL.to_owned()),
                )
                .api_key(
                    args.api_key
                        .clone()
                        .unwrap_or_else(|| etilbudsavis_api::DEFAULT_API_KEY.to_owned()),
                )
                .build()?;
            match client.auth_state().await {
                Ok(state) => {
                    if args.json {
                        return args.output(&serde_json::json!({
                            "authenticated": state.person.is_some(),
                            "person": state.person,
                        }));
                    }
                    match state.person {
                        Some(person) => println!(
                            "signed in as {} ({}{})",
                            person.email.as_deref().unwrap_or("anonymous"),
                            person.id,
                            if person.is_anonymous {
                                ", anonymous"
                            } else {
                                ""
                            }
                        ),
                        None => println!("token present but not accepted"),
                    }
                }
                Err(error) if error.is_auth() => {
                    if args.json {
                        args.output(&serde_json::json!({
                            "authenticated": false,
                            "error": error.to_string(),
                        }))?
                    } else {
                        println!(
                            "token expired; run `etilbudsavis auth anonymous` or `auth login`"
                        );
                    }
                }
                Err(error) => return Err(error.into()),
            }
            Ok(())
        }
        AuthCommand::Anonymous => {
            let client = args.client().await?;
            let session = client.create_anonymous().await?;
            args.session().save(&StoredSession {
                token: session.token,
                email: session.email,
                created_at: Utc::now(),
            })?;
            println!(
                "created an anonymous account in {}",
                args.session().path().display()
            );
            Ok(())
        }
        AuthCommand::Login { email } => {
            let client = args.client().await?;
            let start = client.otp_initialize(email).await?;
            println!("sent a one-time code to {email}");
            println!("otp id: {}", start.otp_id);
            print!("code: ");
            std::io::stdout().flush()?;
            let mut code = String::new();
            std::io::stdin().read_line(&mut code)?;
            let code = code.trim();
            if code.is_empty() {
                bail!("no code entered");
            }
            let session = client.otp_finalize(&start.otp_id, code).await?;
            args.session().save(&StoredSession {
                token: session.token,
                email: session.email.or_else(|| Some(email.clone())),
                created_at: Utc::now(),
            })?;
            println!(
                "signed in; token stored in {}",
                args.session().path().display()
            );
            Ok(())
        }
        AuthCommand::Verify { otp_id, code } => {
            let client = args.client().await?;
            let session = client.otp_finalize(otp_id, code).await?;
            args.session().save(&StoredSession {
                token: session.token,
                email: session.email,
                created_at: Utc::now(),
            })?;
            println!(
                "signed in; token stored in {}",
                args.session().path().display()
            );
            Ok(())
        }
        AuthCommand::Logout => {
            if let Some(token) = args.stored_token() {
                let client = Client::builder().token(token).build()?;
                if let Err(error) = client.terminate().await {
                    eprintln!("server-side sign-out failed: {error}");
                }
            }
            args.session().clear()?;
            println!("signed out");
            Ok(())
        }
        AuthCommand::Consent { state } => {
            let accepted = match state.as_str() {
                "accepted" | "accept" | "true" => true,
                "rejected" | "reject" | "false" => false,
                _ => bail!("state must be `accepted` or `rejected`"),
            };
            let client = args.account().await?;
            let value = client
                .update_person(&PersonUpdate {
                    privacy_policy_consented: Some(accepted),
                    ..PersonUpdate::default()
                })
                .await?;
            args.output(&value)
        }
        AuthCommand::Notifications { kinds } => {
            let client = args.account().await?;
            let value = client
                .update_person(&PersonUpdate {
                    enabled_notifications: Some(kinds.clone()),
                    ..PersonUpdate::default()
                })
                .await?;
            args.output(&value)
        }
        AuthCommand::Country { code } => {
            let client = args.account().await?;
            let value = client
                .update_person(&PersonUpdate {
                    country_code: Some(code.clone()),
                    ..PersonUpdate::default()
                })
                .await?;
            args.output(&value)
        }
    }
}

async fn catalogue(args: &Args, command: &CatalogueCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        CatalogueCommand::List => {
            let groups = client.grouped_fronts().await?;
            if args.json {
                return args.output(&groups);
            }
            for (category, fronts) in &groups {
                let publications: usize = fronts.iter().map(|front| front.publications.len()).sum();
                println!(
                    "{category}\t{} fronts\t{publications} publications",
                    fronts.len()
                );
            }
            Ok(())
        }
        CatalogueCommand::Fronts {
            business,
            category,
            hide_inactive,
        } => {
            use etilbudsavis_api::FrontsQuery;
            let query = FrontsQuery {
                business_slugs: business.clone().map(|slug| vec![slug]),
                category: category.clone(),
                hide_inactive: hide_inactive.then_some(true),
                ..FrontsQuery::default()
            };
            let fronts = client.fronts(&query).await?;
            if args.json {
                return args.output(&fronts);
            }
            for front in &fronts {
                println!(
                    "{}\t{}\t{} publications",
                    front.category.as_deref().unwrap_or("-"),
                    front.business.name,
                    front.publications.len()
                );
            }
            Ok(())
        }
        CatalogueCommand::Publication { id } => {
            let publication = client.publication(id).await?;
            args.output(&publication)
        }
        CatalogueCommand::RelatedPublications {
            business_id,
            location_id,
        } => {
            let publications = client
                .related_publications(business_id, location_id.as_deref())
                .await?;
            if args.json {
                return args.output(&publications);
            }
            for publication in &publications {
                println!(
                    "{}\t{}\t{}..{}",
                    publication.id,
                    publication.label.as_deref().unwrap_or("-"),
                    format_time(publication.valid_from),
                    format_time(publication.valid_until)
                );
            }
            Ok(())
        }
    }
}

async fn business(args: &Args, command: &BusinessCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        BusinessCommand::List { category, limit } => {
            use etilbudsavis_api::BusinessesQuery;
            let businesses = client
                .businesses(&BusinessesQuery {
                    category: category.clone(),
                    limit: Some(*limit),
                    ..BusinessesQuery::default()
                })
                .await?;
            if args.json {
                return args.output(&businesses);
            }
            println!("id\tname\tcategory\tslug");
            for business in &businesses {
                print_business(business);
            }
            Ok(())
        }
        BusinessCommand::Show { slug } => {
            let business = client.business(slug).await?;
            if args.json {
                return args.output(&business);
            }
            println!("{} ({})", business.name, business.public_id);
            if let Some(description) = &business.short_description {
                println!("{description}");
            }
            if let Some(website) = &business.website {
                println!("{website}");
            }
            Ok(())
        }
        BusinessCommand::Description { public_id } => {
            let text = client.business_description(public_id).await?;
            if args.json {
                args.output(&serde_json::json!({ "publicId": public_id, "description": text }))
            } else {
                println!("{text}");
                Ok(())
            }
        }
    }
}

async fn offer(args: &Args, command: &OfferCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        OfferCommand::List {
            business,
            search,
            department,
            sort,
            hide_upcoming,
            limit,
            all,
        } => {
            let business_id = match business {
                Some(value) => Some(resolve_business_id(&client, value).await?),
                None => None,
            };
            let mut query = OffersQuery {
                business_ids: business_id.map(|id| vec![id]),
                search_term: search.clone(),
                department_slug: department.clone(),
                sort: sort.clone().map(|sort| vec![sort]),
                hide_upcoming: hide_upcoming.then_some(true),
                ..OffersQuery::page(if *all { 100 } else { *limit })
            };
            if *all {
                let offers = client.all_offers(query, *limit).await?;
                return args.output(&offers);
            }
            query.pagination = Some(Pagination::first(*limit));
            let offers = client.offers(&query).await?;
            if args.json {
                return args.output(&offers);
            }
            for offer in &offers.data {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    offer
                        .price
                        .map(|price| format!("{price:.2}"))
                        .unwrap_or_else(|| "-".to_owned()),
                    offer
                        .business
                        .as_ref()
                        .map(|business| business.name.as_str())
                        .unwrap_or("-"),
                    offer.name,
                    offer
                        .unit_price
                        .map(|price| format!(
                            "{price:.2}/{}",
                            offer.base_unit.as_deref().unwrap_or("unit")
                        ))
                        .unwrap_or_else(|| "-".to_owned()),
                    format_time(offer.valid_until)
                );
            }
            Ok(())
        }
        OfferCommand::Show { public_id } => {
            let offer = client.offer(public_id).await?;
            args.output(&offer)
        }
        OfferCommand::Products { public_id } => {
            let products = client.offer_products(public_id).await?;
            args.output(&products)
        }
        OfferCommand::Related { public_id } => {
            use etilbudsavis_api::RelatedOffersQuery;
            let offers = client
                .related_offers(&RelatedOffersQuery {
                    public_id: public_id.clone(),
                    sources: Some(vec![
                        "publication".to_owned(),
                        "business_product".to_owned(),
                    ]),
                })
                .await?;
            args.output(&offers)
        }
        OfferCommand::Departments { business_id, limit } => {
            let query = OffersQuery {
                business_ids: Some(vec![business_id.clone()]),
                sort: Some(vec!["score_desc".to_owned()]),
                hide_upcoming: Some(false),
                ..OffersQuery::page(*limit)
            };
            let departments = client.department_offers(&query).await?;
            if args.json {
                return args.output(&departments);
            }
            for department in &departments.data {
                println!(
                    "{}\t{} offers",
                    department.department_slug,
                    department.offers.len()
                );
            }
            Ok(())
        }
    }
}

async fn store(args: &Args, command: &StoreCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        StoreCommand::List {
            business,
            publication,
            offer,
            search,
            limit,
            all,
        } => {
            let business_id = match business {
                Some(value) => Some(resolve_business_id(&client, value).await?),
                None => None,
            };
            let query = StoresQuery {
                business_id,
                publication_id: publication.clone(),
                offer_id: offer.clone(),
                search_term: search.clone(),
                ..StoresQuery::page(if *all { 100 } else { *limit })
            };
            let stores = if *all {
                client.all_stores(query, *limit).await?
            } else {
                client.stores(&query).await?.data
            };
            if args.json {
                return args.output(&stores);
            }
            for store in &stores {
                println!(
                    "{}\t{}\t{}\t{}",
                    store.name,
                    store.street_address.as_deref().unwrap_or("-"),
                    store.city.as_deref().unwrap_or("-"),
                    store.id
                );
            }
            Ok(())
        }
        StoreCommand::Show { id } => {
            let store = client.store(id).await?;
            args.output(&store)
        }
    }
}

async fn search(args: &Args, command: &SearchCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        SearchCommand::Top { limit } => {
            let searches = client.top_searches(*limit).await?;
            if args.json {
                return args.output(&searches);
            }
            for search in &searches.data {
                println!("{}", search.title);
            }
            Ok(())
        }
        SearchCommand::Suggestions { query } => {
            let results = client.suggestions(query).await?;
            if args.json {
                return args.output(&results);
            }
            for result in &results.data {
                println!(
                    "{}\t{}",
                    result.kind.as_deref().unwrap_or("-"),
                    result.title
                );
            }
            Ok(())
        }
        SearchCommand::Business { term } => {
            let result = client.search_business(term).await?;
            args.output(&result)
        }
        SearchCommand::Seo { term } => {
            let result = client.search_seo(term).await?;
            args.output(&result)
        }
        SearchCommand::Offers { query, limit } => {
            let offers = client
                .offers(&OffersQuery {
                    search_term: Some(query.clone()),
                    ..OffersQuery::page(*limit)
                })
                .await?;
            if args.json {
                return args.output(&offers);
            }
            for offer in &offers.data {
                println!(
                    "{}\t{}\t{}",
                    offer
                        .price
                        .map(|price| format!("{price:.2}"))
                        .unwrap_or_else(|| "-".to_owned()),
                    offer
                        .business
                        .as_ref()
                        .map(|business| business.name.as_str())
                        .unwrap_or("-"),
                    offer.name
                );
            }
            Ok(())
        }
    }
}

async fn video(args: &Args, command: &VideoCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        VideoCommand::List { limit, search } => {
            let videos = client.videos(*limit, search.as_deref()).await?;
            if args.json {
                return args.output(&videos);
            }
            for video in &videos {
                println!(
                    "{}\t{}\t{}",
                    video.id,
                    video
                        .author
                        .as_ref()
                        .and_then(|author| author.name.as_deref())
                        .unwrap_or("-"),
                    video.name
                );
            }
            Ok(())
        }
        VideoCommand::Show { id } => {
            let video = client.video(id).await?;
            args.output(&video)
        }
        VideoCommand::Ingredients { id } => {
            let ingredients = client.video_ingredients(id).await?;
            args.output(&ingredients)
        }
        VideoCommand::Liked => {
            let videos = client.liked_videos().await?;
            args.output(&videos)
        }
        VideoCommand::Mine => {
            let videos = client.my_videos().await?;
            args.output(&videos)
        }
        VideoCommand::Like { id } => {
            let client = args.account().await?;
            client.like_video(id).await?;
            println!("liked {id}");
            Ok(())
        }
        VideoCommand::Unlike { id } => {
            let client = args.account().await?;
            client.unlike_video(id).await?;
            println!("unliked {id}");
            Ok(())
        }
    }
}

async fn favorites(args: &Args, command: &FavoritesCommand) -> Result<()> {
    match command {
        FavoritesCommand::List => {
            let client = args.account().await?;
            let favorites = client.favorites().await?;
            if args.json {
                return args.output(&favorites);
            }
            for business in &favorites {
                print_business(business);
            }
            Ok(())
        }
        FavoritesCommand::Add { business } => {
            let client = args.account().await?;
            let public_id = resolve_business_id(&client, business).await?;
            client.create_favorite(&public_id).await?;
            println!("added {public_id}");
            Ok(())
        }
        FavoritesCommand::Remove { business } => {
            let client = args.account().await?;
            let public_id = resolve_business_id(&client, business).await?;
            client.delete_favorite(&public_id).await?;
            println!("removed {public_id}");
            Ok(())
        }
    }
}

async fn lists(args: &Args, command: &ListCommand) -> Result<()> {
    let client = args.account().await?;
    match command {
        ListCommand::List => {
            let lists = client.shopping_lists().await?;
            if args.json {
                return args.output(&lists);
            }
            println!("id\tname\towned\tcollaborators");
            for list in &lists {
                println!(
                    "{}\t{}\t{}\t{}",
                    list.id, list.name, list.owned_by_you, list.collaborator_count
                );
            }
            Ok(())
        }
        ListCommand::Show { id } => {
            let items = client.shopping_list_items(*id).await?;
            if args.json {
                return args.output(&items);
            }
            for item in &items {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    item.client_id,
                    if item.ticked { "x" } else { " " },
                    item.count,
                    item.name,
                    item.estimated_price
                        .map(|price| format!("{price:.2}"))
                        .unwrap_or_else(|| "-".to_owned())
                );
            }
            Ok(())
        }
        ListCommand::Create { name } => {
            let created: CreatedShoppingList = client.create_shopping_list(name).await?;
            if args.json {
                args.output(&created)
            } else {
                println!("{}", created.id);
                Ok(())
            }
        }
        ListCommand::Rename { id, name } => {
            client.rename_shopping_list(*id, name).await?;
            println!("renamed {id}");
            Ok(())
        }
        ListCommand::Delete { id } => {
            client.delete_shopping_list(*id).await?;
            println!("deleted {id}");
            Ok(())
        }
        ListCommand::Leave { id } => {
            client.leave_shopping_list(*id).await?;
            println!("left {id}");
            Ok(())
        }
        ListCommand::Add {
            id,
            name,
            count,
            comment,
            price,
        } => {
            let mut draft = ShoppingListItemDraft {
                client_id: uuid::Uuid::new_v4().to_string(),
                name: name.clone(),
                count: *count,
                ticked: false,
                created_at: timestamp::now(),
                comment: comment.clone(),
                estimated_price: *price,
                business_public_id: None,
                offer: None,
            };
            if let Some(comment) = comment {
                draft.comment = Some(comment.clone());
            }
            client.add_shopping_list_item(*id, &draft).await?;
            println!("added {} to {id}", draft.client_id);
            Ok(())
        }
        ListCommand::Update {
            id,
            item,
            name,
            count,
            ticked,
            comment,
            price,
        } => {
            let items = client.shopping_list_items(*id).await?;
            let current = items
                .into_iter()
                .find(|entry| entry.client_id == *item)
                .with_context(|| format!("no item {item} on list {id}"))?;
            let mut update = ShoppingListItemUpdate::from_item(&current);
            if let Some(name) = name {
                update.name = name.clone();
            }
            if let Some(count) = count {
                update.count = *count;
            }
            if let Some(ticked) = ticked {
                update.ticked = *ticked;
            }
            if let Some(comment) = comment {
                update.short_description = Some(comment.clone());
            }
            if let Some(price) = price {
                update.estimated_price = Some(*price);
            }
            client.update_shopping_list_item(*id, &update).await?;
            println!("updated {item} on {id}");
            Ok(())
        }
        ListCommand::Remove { id, item } => {
            client.remove_shopping_list_item(*id, item).await?;
            println!("removed {item} from {id}");
            Ok(())
        }
        ListCommand::Clear { id } => {
            let items = client.shopping_list_items(*id).await?;
            let ids: Vec<String> = items.into_iter().map(|item| item.client_id).collect();
            client.remove_shopping_list_items(*id, &ids).await?;
            println!("removed {} items from {id}", ids.len());
            Ok(())
        }
        ListCommand::Accept { share_token } => {
            let created = client.accept_shopping_list_invitation(share_token).await?;
            if args.json {
                args.output(&created)
            } else {
                println!("joined list {}", created.id);
                Ok(())
            }
        }
        ListCommand::AutoCategorize { id, item } => {
            let categorized = client.auto_categorize_shopping_list_item(*id, item).await?;
            args.output(&categorized)
        }
    }
}

async fn agents(args: &Args, command: &AgentCommand) -> Result<()> {
    let client = args.account().await?;
    match command {
        AgentCommand::List => {
            let agents = client.offer_agents().await?;
            if args.json {
                return args.output(&agents);
            }
            for agent in &agents {
                println!("{}\t{}", agent.public_id, agent.term);
            }
            Ok(())
        }
        AgentCommand::Create {
            term,
            geohash,
            max_radius,
            include_upcoming,
        } => {
            let created = client
                .create_offer_agent(&etilbudsavis_api::NewOfferAgent {
                    term: term.clone(),
                    geohash: geohash.clone(),
                    sorting: Some("score_desc".to_owned()),
                    max_radius: *max_radius,
                    include_upcoming: *include_upcoming,
                    price_min: None,
                    price_max: None,
                    business_ids: Vec::new(),
                    is_email_notifications_enabled: false,
                    is_push_notifications_enabled: false,
                })
                .await?;
            args.output(&created)
        }
        AgentCommand::Delete { public_id } => {
            client.delete_offer_agent(public_id).await?;
            println!("deleted {public_id}");
            Ok(())
        }
    }
}

async fn location(args: &Args, command: &LocationCommand) -> Result<()> {
    let client = args.client().await?;
    match command {
        LocationCommand::Show => {
            let location = client.location().await?;
            args.output(&location)
        }
        LocationCommand::Lookup {
            latitude,
            longitude,
        } => {
            let info = client.location_info(*latitude, *longitude).await?;
            args.output(&info)
        }
    }
}
