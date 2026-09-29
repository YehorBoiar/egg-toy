Problem statement:

```
(2a + 3x)/3y <= b /\ c + x/y <= d
```

Division is always bad. We want to introduce a rule that would avoid us dividing `x/y`. In conjure-oxide this rule looks like: `x/y ~~> {aux @ aux * y = x}` (rule 1). We basically replace `x/y` with an auxiliary variable `aux` that has a constraint that `aux * y = x`. That way, in our program later on we would be able to just declare aux once and reuse it everywhere.

Now, in the case above to get to the point where we can apply the `x/y ~~> {aux @ aux * y = x}` rule we need to transform the first statement a little bit. We introduce the rule that looks like: `(Num(a)*b + Num(c)*d)/Num(c)*e ~~> (Num(a)*b)/(Num(c)*e) + Num(c)*(d/e)` (rule 2). In our case

```txt
rule 2 application

   ```txt
   1 | (2*a)/(3*y) + 3*(x/y) <= b
   2 | c + x/y <= d
   ```

rule 1 applicaiton

   ```txt
   1 | aux /\ aux * y = x
   2 | 
   3 | (2*a)/(3*y) + 3*(aux) <= b
   4 | c + aux <= d
   ```

```

Now, how do we decide whether we want to apply the "rule 2" in desctructive rule engine? The answer is we either make the "rule 2" a high priority. The problem with "rule 2" being a high priority is that sometimes we would not want to apply it. For example, if we didn't have `c + x/y <= d`, we wouldn't have benefited from applying that rule as we are not going to use the newly created auxiliary variable.


----

## How would we solve that using proposed e-graph rule engine?

I don't know...

I currently am struggling to create a rule that would hold an auxiliary varialbe.