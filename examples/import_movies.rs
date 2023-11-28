use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

use arroy::{DotProduct, KeyCodec, Reader, Writer};
use heed::{DatabaseFlags, EnvOpenOptions, Unspecified};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    let mut args = std::env::args();
    let mut rng = StdRng::seed_from_u64(32);
    let file =
        std::fs::File::open(args.nth(1).expect("Provide the path to the vector file")).unwrap();
    let reader = BufReader::new(file);

    let now = Instant::now();

    let mut vectors: Vec<(u32, Vec<f32>)> = Vec::new();

    // The file look like that
    // === BEGIN vectors ===
    // 0, [0.010056925, -0.0045358953, 0.009904552, 0.0046241777, ..., -0.050245073, 0.021834975, -0.06859673, 0.02483362, 0.01208456, 0.017311502, 0.01912083]
    // === END vectors ===
    for line in reader.lines().skip(1) {
        let line = line.unwrap();
        if line.starts_with("===") {
            break;
        }

        let (id, vector) = line.split_once(',').expect(&line);
        let id: u32 = id.parse().unwrap();

        let vector = vector
            .trim_matches(|c: char| c.is_whitespace() || c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().parse::<f32>().unwrap())
            .collect();

        vectors.push((id, vector));
        assert_eq!(vectors[0].1.len(), vectors.last().unwrap().1.len());
    }

    println!("Took {:?} to parse the file", now.elapsed());

    println!("There are {} vectors", vectors.len());
    let dimensions = vectors[0].1.len();
    println!("Vectors contains {} dimensions", dimensions);
    println!();

    let dir = tempfile::tempdir().unwrap();
    let env = EnvOpenOptions::new()
        .map_size(1024 * 1024 * 1024 * 2) // 2GiB
        .open(dir.path())
        .unwrap();

    // we will open the default unnamed database
    let mut wtxn = env.write_txn().unwrap();
    let database = env
        .database_options()
        .types::<KeyCodec, Unspecified>()
        .flags(DatabaseFlags::INTEGER_KEY)
        .create(&mut wtxn)
        .unwrap();
    let writer = Writer::<DotProduct>::prepare(&mut wtxn, database, 0, dimensions).unwrap();

    println!("Starts to insert document in the database ...");
    let now = Instant::now();
    for (id, vector) in vectors.iter() {
        writer.add_item(&mut wtxn, *id, vector).unwrap();
    }
    let insert = now.elapsed();

    writer.build(&mut wtxn, &mut rng, None).unwrap();
    wtxn.commit().unwrap();

    let build = now.elapsed();
    println!("Took {:?} to insert the vectors", insert);
    println!("Took {:?} to build", build - insert);
    println!("Took {:?} in total to insert and build", build);
    println!();

    let rtxn = env.read_txn().unwrap();

    let query: Vec<f32> = vec![
        -0.016822422,
        -0.021621706,
        0.00019239318,
        0.054372,
        0.034900583,
        -0.011089119,
        0.042128928,
        0.02026509,
        0.0019038923,
        -0.0014809829,
        -0.033832666,
        -0.029640282,
        -0.054234847,
        0.018665258,
        -0.055895746,
        0.0753006,
        0.0061977296,
        0.032228395,
        0.022795584,
        -0.058140032,
        0.026109613,
        -0.029421866,
        0.04866331,
        0.020661665,
        0.017602501,
        0.020653117,
        0.046483666,
        0.042843442,
        -0.045545023,
        -0.0017392042,
        0.012052985,
        -0.0058927303,
        0.032480225,
        0.009872672,
        0.024758337,
        -0.013354463,
        -0.044432696,
        -0.03226193,
        -0.059227727,
        0.0078192735,
        0.013650394,
        0.0031477972,
        0.005877705,
        0.0068786396,
        0.002517114,
        -0.011458909,
        0.008640344,
        0.044904687,
        -0.0047290456,
        -0.012748338,
        -0.048921518,
        0.07827129,
        0.005205742,
        -0.021857478,
        -0.02370976,
        0.041743826,
        -0.016076453,
        -0.011403813,
        -0.025544455,
        -0.0046601044,
        -0.021723151,
        0.007303265,
        -0.0136509575,
        0.0073000537,
        -0.005085544,
        0.04384241,
        -0.018964743,
        0.03818674,
        -0.09198379,
        -0.032043297,
        -0.0067259674,
        0.019887544,
        0.005341308,
        0.0050093965,
        0.054900724,
        -0.020799952,
        0.020495495,
        0.01472667,
        0.019677797,
        0.037550557,
        -0.010920308,
        0.03371257,
        0.0020930816,
        0.03709999,
        -0.036922902,
        -0.049608115,
        0.0154750785,
        0.007696657,
        -0.058294553,
        0.045302838,
        -0.023393214,
        -0.060448237,
        -0.005798211,
        0.053323198,
        0.04070376,
        -0.0028753958,
        0.051668108,
        -0.0069777397,
        0.033418525,
        0.016234992,
        -0.033323497,
        -0.0074829464,
        -0.008664235,
        -0.05547656,
        -0.11400871,
        -0.03518515,
        -0.0056998464,
        0.01812429,
        -0.031799175,
        -0.0073341345,
        -0.06147767,
        -0.003742939,
        -0.004249079,
        -0.013904026,
        -0.00065635156,
        0.09179383,
        0.004267396,
        0.00015509031,
        -0.049766053,
        0.029881846,
        0.10485467,
        -0.03120661,
        0.014043553,
        0.08671136,
        0.059179407,
        0.029454986,
        -0.0122302845,
        0.06451508,
        0.021481989,
        -0.06484224,
        0.018707344,
        0.022344032,
        -0.004790084,
        -0.04705671,
        0.016396629,
        -0.03346155,
        0.0064264126,
        -0.0053360737,
        0.06672058,
        -0.0078784805,
        -0.016174054,
        0.026566355,
        -0.046398066,
        0.0025418145,
        0.046905387,
        -0.020884424,
        -0.051193744,
        -0.031737294,
        -0.009927951,
        0.023741305,
        -0.058117628,
        0.051733956,
        -0.025581324,
        -0.030992776,
        0.008804903,
        0.04388304,
        0.013213721,
        0.004467152,
        -0.04988626,
        0.0069321035,
        0.039403677,
        0.019677948,
        -0.066907056,
        0.018572355,
        0.013511877,
        -0.010518738,
        0.010099771,
        -0.003633823,
        -0.0631501,
        -0.025649378,
        -0.043461364,
        0.0016490245,
        0.064196914,
        0.033599235,
        -0.013222726,
        0.015318823,
        0.0771801,
        -0.0070276,
        -0.031138066,
        0.0055310773,
        -0.09972089,
        0.05066132,
        0.047467627,
        -0.03498512,
        -0.03416252,
        -0.018362196,
        0.040274452,
        -0.031371195,
        0.030042851,
        0.016328678,
        -0.05765591,
        -0.048823263,
        0.054553114,
        -0.02033182,
        0.046627544,
        0.016558101,
        -0.0033715998,
        0.0006232865,
        -0.0065704435,
        0.008104579,
        0.016307961,
        -0.041840676,
        0.048135996,
        -0.018808063,
        -0.036892023,
        -0.0450471,
        0.02718623,
        -0.036660295,
        -0.022694368,
        0.005702901,
        -0.022678563,
        0.0013453028,
        0.07429447,
        -0.034700394,
        -0.032727163,
        0.00596015,
        0.034842487,
        -0.027818438,
        -0.00051779655,
        -0.014468772,
        0.033954486,
        0.04148899,
        -0.0829876,
        -0.015300944,
        0.015376903,
        0.09567573,
        0.036652327,
        -0.049033575,
        -0.04484115,
        0.041701544,
        -0.057027884,
        0.0069984253,
        -0.0053272387,
        0.025826871,
        0.002177651,
        -0.030157669,
        0.007895542,
        -0.014717798,
        0.054724272,
        -0.05034077,
        -0.016694192,
        0.038352106,
        -0.060709346,
        0.08236629,
        -0.0096279215,
        0.014632059,
        0.025158316,
        -0.0009260515,
        -0.043707818,
        -0.01941624,
        -0.0118600605,
        -0.035666965,
        0.037794825,
        0.014687504,
        0.038666032,
        -0.075831376,
        -0.038647566,
        -0.048394937,
        0.031239703,
        0.029136332,
        -0.00076040986,
        -0.015906896,
        0.03718925,
        -0.0140040675,
        -0.037951406,
        -0.041062936,
        -0.06529122,
        0.011906159,
        -0.011368897,
        0.0060307034,
        0.03684682,
        0.031995844,
        -0.033985753,
        -0.018714348,
        -0.012443444,
        -0.007389346,
        0.03257332,
        -0.04580996,
        -0.026579294,
        -0.024290696,
        -0.025647637,
        0.022456668,
        -0.02420987,
        -0.065709755,
        -0.02623659,
        -0.028259972,
        0.019707581,
        -0.022819564,
        -0.0409341,
        0.026851093,
        0.031858675,
        0.048687093,
        -0.013439109,
        0.011736404,
        0.016420575,
        0.03451187,
        -0.0059358296,
        0.015338021,
        0.04402986,
        0.033739056,
        0.033959225,
        0.0068245684,
        -0.0143376645,
        -0.0007635987,
        -0.01949658,
        0.016379116,
        0.018640755,
        -0.06126936,
        -0.22691156,
        0.015514225,
        -0.0010716971,
        0.0044359663,
        0.03258783,
        -0.0018310734,
        0.010761778,
        -0.033404265,
        0.005418415,
        0.028870588,
        -0.0366465,
        0.025508897,
        -0.003327967,
        -0.025249101,
        0.041501254,
        -0.06906739,
        -0.03184493,
        -0.041302733,
        -0.03037772,
        0.015740091,
        0.008446552,
        -0.0459613,
        -0.022405358,
        -0.0036640046,
        0.017842831,
        0.003960712,
        -0.025942408,
        -0.038227286,
        -0.045894515,
        -0.01752483,
        0.017444108,
        -0.051017836,
        0.029609472,
        0.008688325,
        0.020816054,
        0.008120903,
        0.03892946,
        -0.033378396,
        0.02176841,
        0.027685048,
        -0.012064678,
        -0.079198286,
        -0.04271553,
        0.005021753,
        0.066962436,
        -0.03443632,
        -0.004004281,
        -0.050009515,
        -0.01630804,
        0.06379373,
        0.055116866,
        0.027930314,
        0.043325268,
        0.02733439,
        -0.015951807,
        0.059688378,
        -0.0075212875,
        0.03786285,
        -0.04638327,
        -0.043671872,
        0.043587692,
        0.011264745,
        -0.059823193,
        0.008415408,
        -0.040225852,
        -0.05263509,
        -0.0038932117,
        -0.047234535,
        0.05749084,
        0.029582193,
        -0.012869698,
        0.027698075,
        -0.014221754,
        -0.05440618,
        0.007839065,
        -0.028753158,
        -0.029088387,
        -0.00039888048,
        0.012631819,
        0.0038486738,
        -0.059913363,
        -0.0034661351,
        0.011339918,
        0.005836589,
        -0.018044928,
        -0.035229705,
        0.0015524679,
        -0.035521194,
        -0.028409205,
        0.0004174717,
        0.060292065,
        -0.009710763,
        -0.04719587,
        0.034226153,
        0.04258676,
        0.03754591,
        0.056335006,
        -0.012293127,
        0.03885916,
        -0.011872468,
        0.026709288,
        -0.030494772,
        -0.0027441443,
        0.01256448,
        0.0070703924,
        0.011282641,
        -0.03820788,
        -0.029001744,
        0.0024300558,
        -0.0032799696,
        0.037857816,
        0.001686728,
        0.056249045,
        -0.01862739,
        0.04376537,
        -0.0019654054,
        0.050269835,
        0.035223164,
        0.0059567657,
        0.013870472,
        -0.001804614,
        0.027300585,
        -0.03382222,
        -0.041098855,
        -0.060636565,
        0.0047175046,
        0.029142305,
        0.06523361,
        0.028681634,
        -0.023454288,
        -0.018000197,
        -0.030090509,
        -0.0046562785,
        -0.04519735,
        0.047884777,
        -0.00059952086,
        -0.03280122,
        -0.08012555,
        0.008639195,
        0.01629006,
        0.032155965,
        0.034481294,
        0.021274198,
        0.010470909,
        0.022913199,
        -0.035904404,
        0.041294016,
        -0.00987633,
        -0.021613108,
        0.012205929,
        0.005322071,
        0.025864823,
        0.08942025,
        -0.08067831,
        -0.014871667,
        -0.034839284,
        0.028048998,
        -0.0063091223,
        0.037978478,
        -0.055790387,
        0.0045954804,
        -0.042958327,
        0.02137769,
        -0.008589233,
        -0.00062141696,
        0.052822173,
        0.034125473,
        -0.015106767,
        0.0030919765,
        -0.0072712647,
        0.0056287237,
        0.019516133,
        -0.031278323,
        0.025771588,
        0.01701546,
        0.019516064,
        0.016180338,
        0.01349268,
        0.011978184,
        0.011838524,
        -0.0050102035,
        -0.06970658,
        0.022854539,
        -0.004192521,
        0.0577575,
        -0.003954721,
        -0.054374386,
        -0.027609108,
        0.0134023735,
        0.010305641,
        -0.011130317,
        0.052328475,
        0.014928648,
        -0.013976018,
        -0.07100651,
        -0.06789901,
        -0.031873316,
        -0.011598853,
        0.029284442,
        -0.04940027,
        0.0100974385,
        -0.02187546,
        -0.062819175,
        0.0069366414,
        0.052176703,
        -0.06834835,
        0.013463273,
        -0.0013379813,
        0.005786334,
        0.017775143,
        -0.01291353,
        -0.016923305,
        -0.049682386,
        -0.034103107,
        0.010883184,
        -0.055132758,
        0.025268175,
        -0.025599582,
        0.015927013,
        -0.03237898,
        -0.027073668,
        -0.034358867,
        -0.027672807,
        0.022677394,
        -0.03531693,
        0.010573503,
        0.00032215187,
        0.0066956943,
        -0.051510572,
        -0.029456092,
        0.05758612,
        -0.038166363,
        -0.00999853,
        0.05758596,
        -0.006796505,
        0.028503977,
        -0.024184246,
        0.054051045,
        0.0040905816,
        0.099899694,
        0.06076009,
        0.011753628,
        -0.03253187,
        -0.0035343366,
        0.02351163,
        0.03206495,
        0.004892613,
        -0.04530409,
        -0.0056237346,
        -0.006101407,
        0.019704496,
        -0.010228795,
        0.027814431,
        0.020409154,
        0.033115197,
        -0.07446951,
        -0.042142425,
        0.03928483,
        -0.022784598,
        -0.003539396,
        -0.0074683367,
        0.043651864,
        0.007761874,
        0.022063423,
        0.05344986,
        0.05065469,
        0.029476669,
        -0.028968832,
        0.023550583,
        -0.022291148,
        0.055309687,
        -0.053843252,
        0.020895477,
        -0.0148687605,
        0.012166838,
        0.0033556349,
        -0.07022937,
        -0.059401378,
        0.013194393,
        -0.0419862,
        -0.0070434613,
        0.030479655,
        -0.053955454,
        -0.031870224,
        0.034511264,
        -0.047943473,
        0.0069080396,
        0.026099209,
        -0.012516935,
        0.0003174421,
        -0.006716995,
        0.07027558,
        0.038463045,
        -0.016081728,
        0.05018074,
        -0.062176052,
        0.08961092,
        0.03679902,
        0.011107996,
        -0.0032339245,
        -0.0118898135,
        0.013669906,
        0.056221563,
        -0.049234938,
        0.003090264,
        0.01062722,
        -0.008937757,
        -0.08464787,
        -0.032616463,
        0.055935893,
        0.006192905,
        -0.014768529,
        0.04930304,
        0.053852808,
        -0.036349185,
        -0.037947245,
        0.0076732435,
        -0.040889677,
        0.022189876,
        0.015142795,
        0.005928425,
        -0.009679575,
        0.039194115,
        0.0041091475,
        0.035120673,
        0.016776932,
        -0.04100678,
        0.041131947,
        0.040904496,
        0.047341976,
        0.029321635,
        0.030489001,
        -0.0135518275,
        0.038717188,
        0.0017859036,
        -0.044703316,
        -0.007397534,
        0.029149175,
        -0.00021891313,
        0.019795585,
        -0.054424375,
        0.010228703,
        -0.0057461066,
        0.05096695,
        0.05683213,
        -0.018136851,
        -0.0030009004,
        -0.033427265,
        0.010878728,
        0.050759643,
        0.040795874,
        0.019920254,
        -0.026135486,
        -0.07642272,
        0.035290312,
        0.004655317,
        0.0043676766,
        -0.010411962,
        -0.0076723946,
        0.015248613,
        0.008905208,
        -0.0002423048,
        0.03892336,
        -0.025703456,
        -0.021123456,
        -0.00066909986,
        0.04459856,
        0.052217484,
        -0.017885901,
        -0.015303531,
        0.0057848957,
        -0.036129624,
        -0.0736907,
        0.035401847,
        -0.025658514,
        -0.0082354145,
        -0.0012491915,
        -0.040769547,
        -0.039205503,
        0.05530217,
        -0.014954734,
        0.0056790086,
        -0.04454665,
        -0.028425619,
        -0.034654,
        -0.057087515,
        -0.0224583,
        -0.005496095,
        -0.009889468,
        -0.05025576,
        -0.009459795,
        -0.00871503,
        0.021968294,
        0.0074964114,
        -0.032455806,
        -0.005696087,
        0.005180231,
        0.056079067,
        -0.03189999,
        0.045113377,
        0.061360348,
        0.01839327,
        -0.053088665,
        0.04942768,
        0.014662789,
        0.06641078,
        -0.008998172,
        -0.009717696,
        -0.079248,
        0.047506567,
        0.04778238,
        0.025009798,
        -0.03899872,
        0.009850679,
        -0.04399064,
        -0.053494785,
        0.055456433,
        0.026770461,
        -0.011158729,
        -0.073486604,
        -0.04088162,
        -0.023263954,
        -0.022185653,
        0.03401001,
        -0.034742568,
        0.043794204,
        0.004035502,
        0.011585448,
        -0.009235968,
        0.031503983,
        0.016500674,
        -0.012498497,
        -0.05733327,
        0.0024852154,
        -0.02377962,
        -0.072548844,
        -0.008489325,
        0.01825339,
        0.032909963,
        -0.023669574,
        0.0022601841,
        -0.008336443,
        0.0041536367,
        0.007989558,
        -0.035507284,
        -0.03951105,
        0.0069870483,
        0.04283141,
        -0.05102877,
        -0.025309727,
        0.052937508,
        -0.014378752,
        -0.012047669,
        -0.024964543,
        -0.00071902486,
        0.009493713,
        0.024152702,
        0.022622166,
        0.06481285,
        0.0022744837,
    ];

    let reader = Reader::<DotProduct>::open(&rtxn, 0, database).unwrap();

    let now = Instant::now();

    let ret = reader.nns_by_vector(&rtxn, &query, 20, None).unwrap();

    dbg!(ret);

    println!("Louis's query took {:?}", now.elapsed());

    let mut durations = Vec::new();

    println!("Starts querying all documents ...");
    for (id, _) in vectors {
        let now = Instant::now();
        reader.nns_by_item(&rtxn, id, 20, None).unwrap().unwrap();
        durations.push(now.elapsed());
    }
    println!("Making the stats");

    let average = durations.iter().sum::<Duration>() / durations.len() as u32;
    println!("On average it took {:?} to query a vector", average);

    let slowest = durations.iter().max().unwrap();
    println!("The slowest query took {:?}", slowest);

    let slowest = durations.iter().max().unwrap();
    println!("The slowest query took {:?}", slowest);
}
