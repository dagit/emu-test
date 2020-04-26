set datafile separator ','
set key autotitle columnhead
set terminal x11 enhanced font "Times-Roman,20" size 1800,800
set key font "Times-Roman,20"
set tics font "Times-Roman,20"
set object 1 rectangle from screen 0,0 to screen 1,1 fillcolor rgb"#ffffff" behind
plot 'stats.csv' using 1 with lp lw 4, \
              '' using 2 with lp lw 4, \
              '' using 3 with lp lw 4, \
              '' using 4 with lp lw 4
