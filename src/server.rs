use warp::Filter;

use std::string::ToString;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

fn parse_ip(addr: Option<SocketAddr>) -> String 
{
    return if addr.is_none()
    {
        "None".to_string()
    }
    else
    {
        addr.unwrap().to_string()
    }
}

fn is_limited(addr: Option<SocketAddr>, mut requests_from: HashMap<Ipv4Addr, i32>) -> (bool, HashMap<Ipv4Addr, i32>)
{
    println!("Got request from {}", parse_ip(addr));
    if addr.is_none()
    {
        return (true, requests_from)
    }

    let ip = addr.unwrap().ip();
    let mut ipv4: Ipv4Addr = Ipv4Addr::new(0,0,0,0);

    match ip 
    {
        IpAddr::V4(ip4) => {ipv4 = ip4}
        IpAddr::V6(_ip6) => {return (true, requests_from)}
    }

    let requests = if requests_from.contains_key(&ipv4)
    {
        requests_from[&ipv4]
    }
    else 
    {
        println!("New ip");
        requests_from.insert(ipv4, 0);
        0
    };

    println!("Requests total {}", requests);

    if requests >= 3
    {
        (true, requests_from)
    }
    else
    {
        
        let val = if requests < 3
        {
            requests+1
        }
        else 
        {
            3
        };

        *requests_from.get_mut(&ipv4).unwrap() = val;
        (false, requests_from)
    }
}

#[tokio::main]
async fn main() {

    let requests_from: HashMap<Ipv4Addr, i32> = HashMap::new();

    // wrap the hashmap into a Arc object, which may be cloned later (with all clones referencing the origin data)
    //  further wrap in a mutex so each thread can lock it
    //  when locked the mutex will release when going out of scope
    let state = Arc::new(Mutex::new(requests_from));


    let ip_filter = warp::addr::remote()
        .map({
                    let state = state.clone();
                    move |addr:Option<SocketAddr>| -> bool 
                    {
                        let result = is_limited(addr, state.lock().unwrap().clone());
                        *state.lock().unwrap() = result.1;
                        result.0    
                    }
                }
        );

    let routes = warp::any()
        .and(ip_filter)
        .map(|x: bool| 
            {
                if !x 
                {
                    let reponse = "Not blocked";
                    warp::reply::json(&reponse)
                } 
                else 
                {
                    let reponse = "Blocked";
                    warp::reply::json(&reponse)
                }
            }
        );

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await

}