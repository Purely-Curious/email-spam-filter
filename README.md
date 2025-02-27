
# How to use.

### CSV format
* the columns should be text and a number
* the number represents whether or not the message is spam or not, 1 if spam, 0 if ham (not spam).

If there's no provided dataset, it'll run with the internal dataset.

> cargo run < [insert your own email dataset]

However if that isnt the case. Run.
> cargo run or cargo r


Then enter your emails in the form of a csv file after the prompt. 
The provided emails will be classified as either spam or ham (not spam) and written into a two different text files. The spam emails residing in one file and the ham ones in the other.