module Main where

import Control.Monad (replicateM)
import System.Random

data Genome = Genome
  { linesGens :: [Int]
  , sizeGen :: Int
  } deriving (Show)

mkGenome :: IO Genome
mkGenome = do
  linesGens <- randomList (-9, 9) 15
  sizeGen <- randomRIO (2, 12)
  return $ Genome linesGens sizeGen

replace :: [a] -> Int -> a -> [a]
replace lst index elem =
  let (xs, _:ys) = splitAt index lst
   in xs ++ elem : ys

genMutate :: (Int, Int) -> Int -> IO Int
genMutate (bottom, upper) old = do
  deltaB <- randomIO :: IO Bool
  let delta =
        if deltaB
          then -1
          else 1
      newValueUnbounded = old + delta
      newValue = min (max bottom newValueUnbounded) upper
  return newValue

genomeMutate :: Genome -> IO Genome
genomeMutate genome = do
  genIndex <- randomRIO (0, 15) :: IO Int
  if genIndex < 15
    then do
      let oldGen = linesGens genome !! genIndex
      newGen <- genMutate (-9, 9) oldGen
      let newLinesGens = replace (linesGens genome) genIndex newGen
      return $ Genome newLinesGens (sizeGen genome)
    else do
      newGen <- genMutate (2, 12) (sizeGen genome)
      return $ Genome (linesGens genome) newGen

randomList :: (Int, Int) -> Int -> IO [Int]
randomList (bottom, upper) len = replicateM len (randomRIO (bottom, upper))

main = do
  genome <- mkGenome
  newGenome <- genomeMutate genome
  print $ "Old genome: " ++ show genome
  print $ "New genome: " ++ show newGenome
