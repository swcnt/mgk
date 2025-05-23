#![allow(non_snake_case, dead_code, unused_assignments)]
use noisy_float::prelude::*;
use rand::prelude::*;
use rand_distr::Exp;
use rand_distr::Gamma;
use rand_distr::Uniform;
use std::f64::INFINITY;
/*
// statrs has mean and sampling formulas, not needed for now?
use statrs::distribution::Exp as aExp;
use statrs:(:distribution::Continuous as aCon;
use statrs::distribution::Gamma as aGammma;
*/

const EPSILON: f64 = 1e-8;
const DEBUG: bool = false;

fn main() {
    println!("Lambda; Mean Response Time;");

    //let dist = Dist::Hyperexp(1.0,job_size_mu,0.5);
    //let dist = Dist::Gamma(3.0, 0.3);
    //let dist = Dist::Uniform(0.01,1.0);
    let dist = Dist::Expon(1.0);
    let num_servers = 1;
    let num_jobs = 10_000_000;
    let seed = 3;

    //homogenous job service requirement:
    //let job_req_dist = Dist::Constant(0.45);
    let job_req_dist = Dist::Uniform(0.0, 1.0);

    let policy = Policy::DBE;
    println!(
        "Policy : {:?}, Duration: {:?}, Requirement: {:?}, Jobs per data point: {}, Seed: {}",
        policy, dist, job_req_dist, num_jobs, seed
    );
    for lam_base in 1..20 {
        let lambda = lam_base as f64 / 10.0;
        let check = simulate(
            policy,
            num_servers,
            num_jobs,
            dist,
            lambda,
            seed,
            job_req_dist,
        );

        println!("{}; {};", lambda, check);
    }
}
/*
fn simmacro(lambda: f64) -> f64 {


    let check = simulate(
        Policy::FCFS,
        num_servers,
        num_jobs,
        dist,
        arr_lambda,
        seed,
        job_req_dist,
    );
    check



}

*/

#[derive(Debug)]
struct Job {
    arrival_time: f64,
    original_size: f64,
    rem_size: f64,
    service_req: f64,
}

// Make a distribution enum

#[derive(Debug, Clone, Copy)]
enum Dist {
    Expon(f64),
    // i know hyperexp with prob_low = 0 is just an exponential, but i wanna try this :>
    Hyperexp(f64, f64, f64),
    Gamma(f64, f64),
    Uniform(f64, f64),
    Constant(f64),
}

impl Dist {
    fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        match self {
            Dist::Hyperexp(low_mu, high_mu, prob_low) => {
                let mu = if *prob_low == 1.0 {
                    low_mu
                } else if rng.r#gen::<f64>() < *prob_low {
                    low_mu
                } else {
                    high_mu
                };
                Exp::new(*mu).unwrap().sample(rng)
            }
            Dist::Expon(lambda) => Exp::new(*lambda).unwrap().sample(rng),

            Dist::Gamma(k, scale) => Gamma::new(*k, *scale).unwrap().sample(rng),
            Dist::Uniform(low, high) => Uniform::try_from(*low..*high).unwrap().sample(rng),
            Dist::Constant(val) => *val,
        }
    }
    fn mean(&self) -> f64 {
        use Dist::*;
        match self {
            Hyperexp(low_mu, high_mu, prob_low) => prob_low / low_mu + (1.0 - prob_low) / high_mu,
            Expon(lambda) => 1.0 / lambda,
            Gamma(k, scale) => k * scale,
            Uniform(low, high) => (low + high) / 2.0,
            Constant(val) => *val,
        }
    }

    fn meansquare(&self) -> f64 {
        use Dist::*;
        match self {
            Hyperexp(low_mu, high_mu, prob_low) => {
                (2.0 / (low_mu.powf(2.0)) * prob_low)
                    + (2.0 / (high_mu.powf(2.0)) * (1.0 - prob_low))
            }
            Expon(lambda) => 2.0 / lambda.powf(2.0),
            Gamma(k, scale) => ((k + 1.0) * k) / (1.0 / scale).powf(2.0),
            Uniform(low, high) => (1.0 / 3.0) * ((high.powf(3.0) - low.powf(3.0)) / (low - high)),
            Constant(val) => *val,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Policy {
    FCFS,
    PLCFS,
    SRPT,
    FCFSB,
    SRPTB,
    PLCFSB,
    LSF,
    LSFB,
    MSF,
    MSFB,
    SRA,
    SRAB,
    LRA,
    LRAB,
    DB(usize),
    DBE,
}

impl Policy {
    // return whichever criterion jobs get sorted by.

    fn index(&self, job: &Job) -> f64 {
        match self {
            Policy::FCFS | Policy::FCFSB => job.arrival_time,
            Policy::PLCFS | Policy::PLCFSB => -job.arrival_time,
            Policy::SRPT | Policy::SRPTB => job.rem_size,
            Policy::LSF | Policy::LSFB => job.service_req,
            Policy::MSF | Policy::MSFB => -job.service_req,
            Policy::SRA | Policy::SRAB => job.rem_size * job.service_req,
            Policy::LRA | Policy::LRAB => -job.rem_size * job.service_req,
            Policy::DB(_) | Policy::DBE => job.arrival_time,
        }
    }
}

fn fcfstest(arr_lambda: f64, size_dist: &Dist) {
    let avg_size = size_dist.mean();

    // rho -- must be less than 1
    let rho = arr_lambda * avg_size;

    let esquare = size_dist.meansquare();

    // we have everything needed to find E[T] and E[N]
    let ET = (arr_lambda * esquare) / (2.0 * (1.0 - rho)) + avg_size;
    let EN = ET * arr_lambda;
    if DEBUG {
        println!("Mean Response time is: {}, Mean Queue Length is {}", ET, EN);
    }
}


fn qscan(vec: &Vec<Job>, num_servers: usize) -> usize {
    let mut index = 0;
    let total_resource = num_servers as f64;
    if DEBUG {
        println!("Total resource = {}", total_resource);
        println!("Queue length={}", vec.len());
    }
    // count how much "size" we have remaining in this timestep
    let mut taken_service: f64 = EPSILON;

    // very naive while loop
    while taken_service < total_resource {
        if index >= vec.len() {
            if DEBUG {
                println!("Max Length reached");
            }
            return vec.len();
        };
        taken_service = taken_service + vec[index].service_req;
        index = index + 1;
    }
    index - 1
}

fn take_to_vec(num_take: usize) -> Vec<usize> {
    let v: Vec<usize> = (0..num_take).collect();
    v
}

fn backfill(vec: &Vec<Job>, num_servers: usize) -> Vec<usize> {
    let total_resource = num_servers as f64;
    if DEBUG {
        println!("Backfilling up to {}", total_resource);
    }

    // initialize the taken_resource counter, loop with a skip
    let mut taken_resource = 0.0;
    let mut indices: Vec<usize> = vec![];

    for ii in 0..vec.len() {
        let trial_take = taken_resource + vec[ii].service_req;
        if trial_take > total_resource {
            continue;
        }
        if trial_take + EPSILON <= total_resource {
            taken_resource = trial_take;
            indices.push(ii);
        }
    }
    indices
}

fn eval_buckets(vec: &Vec<Job>, k: usize, upper: f64, lower: f64) -> Vec<usize> {
    assert!(k % 2 == 1);

    let increment = (upper - lower) / (k as f64);
    if DEBUG {
        println!("Increment is {}, k is {}",increment,k);
    }
    let all_indices: Vec<usize> = (0..vec.len()).collect();
    let bucket_numbers: Vec<usize> = all_indices
        .iter()
        .map(|index| (vec[*index].service_req / increment).floor() as usize)
        .collect();
    
    // evaluate bucket scores
    let mut bucket_counts: Vec<f64> = vec![0.0; k];
    for ii in 0..vec.len() {
        bucket_counts[bucket_numbers[ii]] += 1.0;
    }
    // square all bucket scores
    let bucket_scores: Vec<f64> = bucket_counts.iter().map(|score| score.powf(2.0)).collect();
    
    // compare bucket scores and return the highest one
    let mut target = 0; // 0 corresponds to bucket pair 0,k-1
    let mut temp_new = 0.0;
    let mut sitting_best = 0.0;
    for jj in 0..((bucket_scores.len()-1)/2) {
        temp_new = bucket_scores[jj] + bucket_scores[k-jj-2];
        
        if temp_new > sitting_best + EPSILON {
            sitting_best = temp_new;
            target = jj; // assign target var
        }
    }

    // check the last bucket
    let mut last = false;
    if bucket_scores[k-1] > sitting_best {
        target = k-1;
        last = true;
    }

    if DEBUG {
        println!("Bucket scores: {:?}",bucket_scores);
        println!("Bucket numbers of jobs: {:?}",bucket_numbers);
        println!("Last bucket targeted?: {:?}",last);
    }
    let mut ret_indices: Vec<usize> = vec![];
    
    // fetch the indices of the jobs corresponding to the winning bucket
    for kk in 0..vec.len() {
        
        if bucket_numbers[kk] == target {
            ret_indices.push(kk);
            break;
        }
    }


    for kk in 0..vec.len() {
        if !last {
            if bucket_numbers[kk] == k-target-2 {
                ret_indices.push(kk);
                break;
            }
        }
    }

/*

    for kk in 0..vec.len() {
       
       if ((bucket_numbers[kk] == target) | (bucket_numbers[kk] == k-target-2)) & !last {
           ret_indices.push(kk);
       }
       if last {
           if bucket_numbers[kk] == target {
               ret_indices.push(kk);
           }
       }
    }
    */

    if DEBUG {
    println!("Working on jobs {:?}",ret_indices);
    }
    ret_indices

}

fn lambda_to_k(lambda: f64) -> usize {
    let k_mid = (lambda + 2.0) / (2.0 - lambda);
    let mut attempt_k = k_mid.ceil() as usize;
    if attempt_k % 2 == 0 {
        attempt_k = attempt_k + 1
    }
    attempt_k as usize
}

fn queue_indices(vec: &Vec<Job>, num_servers: usize, policy: Policy, lambda: f64) -> Vec<usize> {
    let l_lim = 0.0;
    let u_lim = num_servers as f64;
    match policy {
        Policy::FCFS => take_to_vec(qscan(vec, num_servers)),
        Policy::PLCFS => take_to_vec(qscan(vec, num_servers)),
        Policy::SRPT => take_to_vec(qscan(vec, num_servers)),
        Policy::FCFSB => backfill(vec, num_servers),
        Policy::SRPTB => backfill(vec, num_servers),
        Policy::PLCFSB => backfill(vec, num_servers),
        Policy::LSF => take_to_vec(qscan(vec, num_servers)),
        Policy::MSF => take_to_vec(qscan(vec, num_servers)),
        Policy::LSFB => backfill(vec, num_servers),
        Policy::MSFB => backfill(vec, num_servers),
        Policy::SRA => take_to_vec(qscan(vec, num_servers)),
        Policy::LRA => take_to_vec(qscan(vec, num_servers)),
        Policy::SRAB => backfill(vec, num_servers),
        Policy::LRAB => backfill(vec, num_servers),
        Policy::DB(k) => eval_buckets(vec,k,u_lim,l_lim),
        Policy::DBE =>  eval_buckets(vec,lambda_to_k(lambda),u_lim,l_lim),
    }
}

fn simulate(
    policy: Policy,
    num_servers: usize,
    num_jobs: u64,
    dist: Dist,
    arr_lambda: f64,
    seed: u64,
    req_dist: Dist,
) -> f64 {
    let mut num_completions = 0;
    let mut queue: Vec<Job> = vec![];
    let mut total_response = 0.0;
    let mut time = 0.0;
    let mut rng = StdRng::seed_from_u64(seed);
    let arrival_dist = Exp::new(arr_lambda).unwrap();
    let mut total_work = 0.0;
    let mut num_arrivals = 0;

    // predict what outcome should be (if fcfs):
    if DEBUG {
        fcfstest(arr_lambda, &dist);
    }

    // initialize a first job arrival
    let mut next_arrival_time = arrival_dist.sample(&mut rng);

    while num_completions < num_jobs {
        queue.sort_by_key(|job| n64(policy.index(job)));
        if queue.len() > num_jobs.isqrt() as usize {
            println!("Error: queue length past threshold");
            break;
        }
        // i'll test policies later once FCFS metrics are confirmed (wow they are now)
        if DEBUG {
            println!(
                "Time is {}: | Queue: {:?} | Current work: {} Total work: {}",
                time,
                queue,
                queue.iter().map(|job| job.rem_size).sum::<f64>(),
                total_work,
            );
            std::io::stdin()
                .read_line(&mut String::new())
                .expect("whatever");
            // find next event (arrival or completion)
            // next_completion is NOT a time, it is a duration
        }

        // determine how many jobs need to get worked on in the sorted queue.
        //let num_workable = qscan(&queue, num_servers);
        //
        let mut index_workable = queue_indices(&queue, num_servers, policy, arr_lambda);
        index_workable.sort();

        if DEBUG {
            println!("{:?} jobs eligible for work.", index_workable);
        }

        let capacity: f64 = index_workable.iter().map(|index| queue[*index].service_req).sum();
        assert!(capacity < 1.0 + EPSILON);

        let next_completion = index_workable
            .iter()
            .map(|index| queue[*index].rem_size)
            .min_by_key(|f| n64(*f))
            .unwrap_or(INFINITY);

        //find next completion time out of eligible jobs
        /*
        let next_completion = queue
            .iter()
            .take(num_workable)
            .map(|job| job.rem_size as f64)
            .min_by_key(|f| n64(*f))
            .unwrap_or(INFINITY);
        */
        let timestep = next_completion.min(next_arrival_time - time);
        let was_arrival = timestep < next_completion;

        // time moves forward
        time += timestep;

        // all jobs currently in service get worked on
        /*
        queue
            .iter_mut()
            .take(num_workable) // or just 1 for now
            .for_each(|job| job.rem_size -= timestep as f64);
        */

        index_workable
            .iter()
            .for_each(|index| queue[*index].rem_size -= timestep);

        // Remove jobs that may have finished (only the first num_servers) jobs
        // in the queue need to be checked.
        // this is so smart izzy what
        // i dont know why they reverse here though

        for &index in index_workable.iter().rev() {
            assert!(index < queue.len());
            if queue[index].rem_size < EPSILON {
                let job = queue.remove(index);
                total_response += time - job.arrival_time;
                num_completions += 1;
            }
        }
        /*
        for i in (0.. .min(queue.len())).rev() {
            if queue[i].rem_size < EPSILON {
                let job = queue.remove(i);
                total_response += time - job.arrival_time;
                num_completions += 1;
            }
        }
        */
        // if the job was an arrival, tick up the total work in the queue (sum of rem_sizes)
        // and add a new job to the queue.

        if was_arrival {
            total_work += queue.iter().map(|job| job.rem_size).sum::<f64>();
            num_arrivals += 1;
            let new_job_size = dist.sample(&mut rng);
            let new_service_req = req_dist.sample(&mut rng);
            let new_job = Job {
                rem_size: new_job_size,
                original_size: new_job_size,
                arrival_time: time,
                service_req: new_service_req,
            };
            queue.push(new_job);
            next_arrival_time = time + arrival_dist.sample(&mut rng);
        }
    }

    // report mean queue load
    //total_work / num_arrivals as f64
    //OR report mean response time
    total_response / num_arrivals as f64
}
