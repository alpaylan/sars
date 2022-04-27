# SARS:  Lightweight Suffix Arrays for Rust

## Introduction

I implemented the project in Rust, using `clap` for argument handling, 
`rust-bio` for suffix arrays, 
`bincode/serde` for serialization, 
`rustc-hash` for _FxHash_ 
functions and `bstr` for
_byte/string_ conversions. 

Github Link: [https://github.com/alpaylan/sars](https://github.com/alpaylan/sars)

Crates.io Link: [https://crates.io/crates/sars](https://crates.io/crates/sars)



## Build

**-- What did you find to be the most challenging part of implementing the buildsa program?**  
I had to change my implementation that used Rust String constructs to Vec<u8> constructs, because the first implementation took an enormous time because of allocations I needed to do for substring equality check. It was the most exhausting part of the building process.

**-- For references of various size:**  
**--- How long does building the suffix array take?**  
For the given _ecoli_ data, it takes around 13 seconds to build the suffix array. We can see that it's growth function approximates $O(N)$ as each halving in the size corresponds to an halving in the time.

| Reference Size(As Bytes) | Time(As Milliseconds) |
|--------------------------| ------- |
| 4639676                  |  12536 |
| 2319838                  |  6185 |
| 1159919                  |  3092 |
|  579960                  |  1531 |
 |  289980                  |  752 |
 |  144990                  |  374 |
 |  72495                   |  188 |
 |  36248                   |  93 |
 |  18124                   |  45 |
 |  9062                    |  22 |
 |  4531                    |  10 |
 |  2266                    |  5 |
 |  1133                    |  2 |
 |  567                     |  1 |

    Table of reference size/time for different sizes. All measurements are done by cutting the given ecoli data by half each time


**--- How large is the resulting serialized file?**  
Size of the suffix array is directly proportional to the size of the reference. When we halve the reference size, suffix array size drops at the same order. |

**--- For the times and sizes above, how do they change if you use the --preftab option with some different values of k?**  
We can see that up until prefix length 7, prefix table is merely negligible. But due to exponential growth, we see a very quick rise from that point on.

| Prefix Length | Resulting Serialized File for Full Ecoli Index |
|---------------| --- |
| None          | 42M   |
| 1             | 42M   |
| 2             | 42M   |
| 3             | 42M   |
| 4             | 42M   |
| 5             | 42M   |
| 6             | 42M   |
| 7             | 42M   |
| 8             | 44M   |
| 9             | 50M   |
| 10            | 72M   |
| 11            | 119M  |
| 12            | 167M  |
| 13            | 196M  |
| 14            | 210M  |
|  15           | 218M  |

    Table of prefix length/file sizes


**-- Given the scaling above, how large of a genome do you think you could construct the suffix array for on a machine with 32GB of RAM, why?**   
We have a 42MB serialized file for a roughly 4MB reference. Hence, the ratio is close to 1/10. Without a prefix table, we could scale up to a 3GB size reference, approximately 1000 times our current reference, which would make our length 4.6B nucleotides.

## Query
**-- What did you find to be the most challenging part of implementing the query program?**  
I changed my data representation halfway through the project for efficiency reasons, which resulted in various bugs in my \texttt{longest\_common\_prefix} and \texttt{simpaccel\_search} functions; I have spent a fair amount of time debugging and solving these bugs. The indirection of using an offset over the reference instead of dealing with actual strings makes it much harder to debug because data is inherently hidden; it requires extra work to construct it.

**--- For references of various size:**  
**--- How long does query take on references of different size, and on queries of different length?**  

| Reference Size(As Bytes) | Time(As Seconds) | 
|--------------------------| --- |
 | 4639676                  | 66 |
 | 2319838                  | 35 |
 | 1159919                  | 18 |
 | 579960                   | 7 |
 | 289980                   | 3 |
 | 144990                   | 2 |
 | 72495                    | 1 |

    Time for the naive algorithm to run on reference
    

**--- How does the speed of the naive lookup algorithm compare to the speed of the simpleaccel lookup algorithm?**  
**-- How does the speed further compare when not using a prefix lookup table versus using a prefix lookup table with different values of k?**

| Prefix Length  | Naive(1) | Naive(2) | Simpaccel(1) | Simpaccel(2) |
|----------------| --- | --- | --- | --- |
| None           | 67 | 67 | 2 | 2 |
| 1              | 31 | 32 | 1 | 1 |
| 2              | 46 | 48 | 2 | 2 |
| 3              | 53 | 54 | 2 | 2 |
| 4              | 41 | 40 | 1 | 1 |
| 5              | 38 | 37 | 1 | 1 |
| 6              | 42 | 34 | 1 | 1 |
 | 7              | 20 | 21 | 0 | 0 |
 | 8              | 16 | 16 | 0 | 0 |
 | 9              | 10 | 10 | 0 | 0 |
 | 10             | 8 | 8 | 0 | 0 |
 | 11             | 5 | 5 | 0 | 0 |
 | 12             | 4 | 4 | 0 | 0 |
 | 13             | 3 | 3 | 0 | 0 |
 | 14             | 3 | 3 | 0 | 0 |
 | 15             | 3 | 3 | 0 | 0|
    
    Time(in seconds) to do 10 queries for Ecoli Data
    


**-- Given the scaling above, and the memory requirements of each type of index, what kind of tradeoff do you personally think makes sense in terms of using more memory in exchange for faster search.** 

We can see the effect of diminishing returns around 7-10 for prefix length. As we can see also see that size of prefix length is negligible up until 7, I think it makes sense to keep it in that region depending on our time and space requirements. 

