use rand::Rng;

pub const FIRST_NAMES: &[&str] = &[
    "James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda",
    "David", "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
    "Thomas", "Sarah", "Christopher", "Karen", "Charles", "Lisa", "Daniel", "Nancy",
    "Matthew", "Betty", "Anthony", "Margaret", "Mark", "Sandra", "Donald", "Ashley",
    "Steven", "Kimberly", "Paul", "Emily", "Andrew", "Donna", "Joshua", "Michelle",
    "Kenneth", "Carol", "Kevin", "Amanda", "Brian", "Dorothy", "George", "Melissa",
    "Timothy", "Deborah", "Ronald", "Stephanie", "Edward", "Rebecca", "Jason", "Sharon",
    "Jeffrey", "Laura", "Ryan", "Cynthia", "Jacob", "Kathleen", "Gary", "Amy",
    "Nicholas", "Angela", "Eric", "Shirley", "Jonathan", "Anna", "Stephen", "Brenda",
    "Larry", "Pamela", "Justin", "Emma", "Scott", "Nicole", "Brandon", "Helen",
    "Benjamin", "Samantha", "Samuel", "Katherine", "Raymond", "Christine", "Gregory", "Debra",
    "Frank", "Rachel", "Alexander", "Carolyn", "Patrick", "Janet", "Jack", "Catherine",
    "Dennis", "Maria", "Jerry", "Heather", "Tyler", "Diane", "Aaron", "Ruth",
    "Jose", "Julie", "Nathan", "Olivia", "Henry", "Joyce", "Peter", "Virginia",
    "Douglas", "Victoria", "Zachary", "Kelly", "Adam", "Lauren", "Kyle", "Christina",
    "Noah", "Joan", "Ethan", "Evelyn", "Jeremy", "Judith", "Walter", "Megan",
    "Christian", "Andrea", "Keith", "Cheryl", "Roger", "Hannah", "Terry", "Jacqueline",
    "Austin", "Martha", "Sean", "Gloria", "Gerald", "Teresa", "Carl", "Ann",
    "Harold", "Sara", "Dylan", "Madison", "Arthur", "Frances", "Lawrence", "Kathryn",
    "Jordan", "Janice", "Jesse", "Jean", "Bryan", "Abigail", "Billy", "Alice",
    "Bruce", "Judy", "Gabriel", "Sophia", "Joe", "Grace", "Logan", "Denise",
    "Albert", "Amber", "Willie", "Doris", "Alan", "Marilyn", "Eugene", "Danielle",
    "Russell", "Beverly", "Vincent", "Isabella", "Philip", "Theresa", "Bobby", "Diana",
    "Johnny", "Natalie", "Bradley", "Brittany", "Roy", "Charlotte", "Ralph", "Marie",
    "Louis", "Kayla", "Randy", "Alexis", "Wayne", "Lori",
];

pub const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas",
    "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson", "White",
    "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson", "Walker", "Young",
    "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores",
    "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell",
    "Carter", "Roberts", "Gomez", "Phillips", "Evans", "Turner", "Diaz", "Parker",
    "Cruz", "Edwards", "Collins", "Reyes", "Stewart", "Morris", "Morales", "Murphy",
    "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson", "Bailey",
    "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward", "Richardson",
    "Watson", "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray", "Mendoza",
    "Ruiz", "Hughes", "Price", "Alvarez", "Castillo", "Sanders", "Patel", "Myers",
    "Long", "Ross", "Foster", "Jimenez", "Powell", "Jenkins", "Perry", "Russell",
    "Sullivan", "Bell", "Coleman", "Butler", "Henderson", "Barnes", "Gonzales", "Fisher",
    "Vasquez", "Simmons", "Griffin", "Marshall", "Owens", "Harrison", "Dean", "Fernandez",
    "Patterson", "Hamilton", "Graham", "Reynolds", "Herrera", "Medina", "Ford", "Spencer",
    "Hunt", "Stone", "Knight", "Burns", "Black", "Palmer", "Walsh", "Weber",
    "Soto", "Gordon", "Dunn", "Dixon", "Freeman", "Webb", "Gibson", "Holmes",
    "Elliott", "Ferguson", "Chambers", "Hicks", "Grant", "Hart", "Fox", "Warner",
    "Rice", "Franklin", "Cunningham", "Craig", "Wagner", "Lane", "Bradley", "Carr",
    "Harvey", "Duncan", "Armstrong", "Berry", "Hudson", "Sullivan", "Greene", "Lawrence",
    "Hawkins", "Wells", "Perkins", "Little", "Stanley", "Pierce", "Weaver", "Stephens",
    "Schmidt", "Harper", "Payne", "Fuller", "Daniels", "Carroll", "Mcdonald", "Mills",
    "Warren", "Austin", "Peters", "Kelley", "Franklin", "Lawson", "Fields", "Gutierrez",
    "Ryan", "Schmidt", "Carr", "Vasquez", "Castillo", "Wheeler", "Chapman", "Oliver",
];

pub const STREET_NAMES: &[&str] = &[
    "Oak", "Main", "Cedar", "Elm", "Maple", "Pine", "Washington", "Lake",
    "Hill", "Walnut", "Spring", "Park", "Sunset", "Highland", "River", "Meadow",
    "Forest", "Cherry", "Willow", "Rose", "Lincoln", "Church", "Broadway", "Jackson",
    "Franklin", "Jefferson", "Adams", "Madison", "Monroe", "Harrison", "Center", "Pleasant",
    "Valley", "Ridge", "College", "Mill", "Academy", "Bridge", "School", "Union",
    "Liberty", "Market", "Garden", "Vine", "Summit", "Prospect", "Court", "Laurel",
    "Magnolia", "Birch", "Poplar", "Ash", "Dogwood", "Hickory", "Spruce", "Sycamore",
    "Holly", "Ivy", "Fern", "Aspen", "Cypress", "Palm", "Sierra", "Canyon",
    "Vista", "Horizon", "Crescent", "Sterling", "Windsor", "Oakwood", "Greenwood", "Fairview",
    "Lakeview", "Woodland", "Brookside", "Hillside", "Riverside", "Parkway", "Northview", "Southgate",
    "Westwood", "Eastgate", "Heritage", "Colonial", "Patriot", "Pioneer", "Legacy", "Beacon",
    "Harbor", "Lighthouse", "Bayview", "Oceanview", "Seaside", "Marina", "Anchor", "Coral",
    "Eagle", "Hawk", "Falcon", "Dove", "Robin", "Cardinal",
];

pub const STREET_SUFFIXES: &[&str] = &[
    "St", "Ave", "Blvd", "Dr", "Ln", "Ct", "Way", "Pl", "Rd",
];

pub const CITIES: &[&str] = &[
    "New York", "Los Angeles", "Chicago", "Houston", "Phoenix", "Philadelphia",
    "San Antonio", "San Diego", "Dallas", "San Jose", "Austin", "Jacksonville",
    "Fort Worth", "Columbus", "Charlotte", "Indianapolis", "San Francisco", "Seattle",
    "Denver", "Nashville", "Oklahoma City", "El Paso", "Washington", "Boston",
    "Las Vegas", "Portland", "Memphis", "Louisville", "Baltimore", "Milwaukee",
    "Albuquerque", "Tucson", "Fresno", "Mesa", "Sacramento", "Atlanta",
    "Kansas City", "Colorado Springs", "Raleigh", "Omaha", "Miami", "Long Beach",
    "Virginia Beach", "Oakland", "Minneapolis", "Tampa", "Tulsa", "Arlington",
    "New Orleans", "Wichita", "Cleveland", "Bakersfield", "Aurora", "Anaheim",
    "Honolulu", "Santa Ana", "Riverside", "Corpus Christi", "Lexington", "Stockton",
    "Pittsburgh", "Cincinnati", "Anchorage", "Henderson", "Greensboro", "Plano",
    "Newark", "Lincoln", "Orlando", "Irvine", "Toledo", "Jersey City",
    "Chula Vista", "Durham", "Laredo", "Madison", "Chandler", "Buffalo",
    "Lubbock", "Scottsdale", "Reno", "Glendale", "Gilbert", "Winston-Salem",
    "North Las Vegas", "Norfolk", "Chesapeake", "Garland", "Irving", "Hialeah",
    "Fremont", "Boise", "Richmond", "Baton Rouge", "Spokane", "Des Moines",
    "Tacoma", "San Bernardino", "Modesto", "Fontana",
];

pub const STATES: &[(&str, &str)] = &[
    ("AL", "Alabama"), ("AK", "Alaska"), ("AZ", "Arizona"), ("AR", "Arkansas"),
    ("CA", "California"), ("CO", "Colorado"), ("CT", "Connecticut"), ("DE", "Delaware"),
    ("FL", "Florida"), ("GA", "Georgia"), ("HI", "Hawaii"), ("ID", "Idaho"),
    ("IL", "Illinois"), ("IN", "Indiana"), ("IA", "Iowa"), ("KS", "Kansas"),
    ("KY", "Kentucky"), ("LA", "Louisiana"), ("ME", "Maine"), ("MD", "Maryland"),
    ("MA", "Massachusetts"), ("MI", "Michigan"), ("MN", "Minnesota"), ("MS", "Mississippi"),
    ("MO", "Missouri"), ("MT", "Montana"), ("NE", "Nebraska"), ("NV", "Nevada"),
    ("NH", "New Hampshire"), ("NJ", "New Jersey"), ("NM", "New Mexico"), ("NY", "New York"),
    ("NC", "North Carolina"), ("ND", "North Dakota"), ("OH", "Ohio"), ("OK", "Oklahoma"),
    ("OR", "Oregon"), ("PA", "Pennsylvania"), ("RI", "Rhode Island"), ("SC", "South Carolina"),
    ("SD", "South Dakota"), ("TN", "Tennessee"), ("TX", "Texas"), ("UT", "Utah"),
    ("VT", "Vermont"), ("VA", "Virginia"), ("WA", "Washington"), ("WV", "West Virginia"),
    ("WI", "Wisconsin"), ("WY", "Wyoming"),
];

pub const EMAIL_DOMAINS: &[&str] = &[
    "gmail.com", "yahoo.com", "outlook.com", "hotmail.com", "aol.com",
    "icloud.com", "mail.com", "protonmail.com", "zoho.com", "yandex.com",
    "gmx.com", "fastmail.com", "tutanota.com", "live.com", "msn.com",
];

pub fn random_string(len: usize, charset: &str, custom: &str) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = match charset {
        "hex" => "0123456789abcdef".chars().collect(),
        "alpha" => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
        "numeric" => "0123456789".chars().collect(),
        "custom" if !custom.is_empty() => custom.chars().collect(),
        _ => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect(),
    };
    (0..len).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn random_number(min: i64, max: i64, decimal: bool) -> String {
    let mut rng = rand::thread_rng();
    if decimal {
        let val: f64 = rng.gen_range(min as f64..=max as f64);
        format!("{:.2}", val)
    } else {
        let val: i64 = rng.gen_range(min..=max);
        val.to_string()
    }
}

pub fn random_email() -> String {
    let mut rng = rand::thread_rng();
    let first = FIRST_NAMES[rng.gen_range(0..FIRST_NAMES.len())].to_lowercase();
    let last = LAST_NAMES[rng.gen_range(0..LAST_NAMES.len())].to_lowercase();
    let domain = EMAIL_DOMAINS[rng.gen_range(0..EMAIL_DOMAINS.len())];
    let num: u32 = rng.gen_range(1..999);
    format!("{}.{}{:0>2}@{}", first, last, num, domain)
}

pub fn random_first_name() -> String {
    let mut rng = rand::thread_rng();
    FIRST_NAMES[rng.gen_range(0..FIRST_NAMES.len())].to_string()
}

pub fn random_last_name() -> String {
    let mut rng = rand::thread_rng();
    LAST_NAMES[rng.gen_range(0..LAST_NAMES.len())].to_string()
}

pub fn random_full_name() -> String {
    format!("{} {}", random_first_name(), random_last_name())
}

pub fn random_street_address() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(100..9999);
    let street = STREET_NAMES[rng.gen_range(0..STREET_NAMES.len())];
    let suffix = STREET_SUFFIXES[rng.gen_range(0..STREET_SUFFIXES.len())];
    format!("{} {} {}", num, street, suffix)
}

pub fn random_city() -> String {
    let mut rng = rand::thread_rng();
    CITIES[rng.gen_range(0..CITIES.len())].to_string()
}

pub fn random_state() -> String {
    let mut rng = rand::thread_rng();
    STATES[rng.gen_range(0..STATES.len())].0.to_string()
}

pub fn random_state_full() -> String {
    let mut rng = rand::thread_rng();
    STATES[rng.gen_range(0..STATES.len())].1.to_string()
}

pub fn random_zip() -> String {
    let mut rng = rand::thread_rng();
    format!("{:05}", rng.gen_range(10000..99999u32))
}

pub fn random_phone() -> String {
    let mut rng = rand::thread_rng();
    let area: u32 = rng.gen_range(200..999);
    let mid: u32 = rng.gen_range(200..999);
    let last: u32 = rng.gen_range(1000..9999);
    format!("({}) {}-{}", area, mid, last)
}

pub fn random_date(format: &str, min_str: &str, max_str: &str) -> String {
    let mut rng = rand::thread_rng();
    let min_ts = if min_str.is_empty() {
        // Default: 1990-01-01
        631152000i64
    } else {
        chrono::NaiveDate::parse_from_str(min_str, "%Y-%m-%d")
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc().timestamp())
            .unwrap_or(631152000)
    };
    let max_ts = if max_str.is_empty() {
        chrono::Utc::now().timestamp()
    } else {
        chrono::NaiveDate::parse_from_str(max_str, "%Y-%m-%d")
            .map(|d| d.and_hms_opt(23, 59, 59).unwrap_or_default().and_utc().timestamp())
            .unwrap_or_else(|_| chrono::Utc::now().timestamp())
    };
    let ts = rng.gen_range(min_ts..=max_ts);
    let fmt = if format.is_empty() { "%Y-%m-%d" } else { format };
    if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
        dt.format(fmt).to_string()
    } else {
        String::new()
    }
}
