#!/bin/sh

git describe --long | sed 's/-/.r/;s/-/./'
